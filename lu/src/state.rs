use std::{cell::RefCell, ffi, marker::PhantomData, ptr::NonNull};

use crate::{
    Config, Library, LuauAllocator, Stack, Thread, ThreadData, ThreadMain, ThreadRef, Userdata,
};

pub struct State<C: Config> {
    libraries: Vec<(&'static str, Library<C>)>,
    alloc: NonNull<C::Allocator>,
    main: NonNull<RefCell<C::MainData>>,
    ptr: NonNull<sys::lua_State>,
}

impl<C: Config> Drop for State<C> {
    fn drop(&mut self) {
        unsafe {
            sys::lua_close(self.ptr.as_ptr());
            self.main.drop_in_place();
            self.alloc.drop_in_place();
        }
    }
}

impl<C: Config> State<C> {
    pub fn new(main_data: C::MainData, alloc: C::Allocator) -> Self {
        let main = NonNull::new(Box::into_raw(Box::new(RefCell::new(main_data)))).unwrap();
        let alloc = NonNull::new(Box::into_raw(Box::new(alloc))).unwrap();

        extern "C-unwind" fn alloc_fn<Alloc: LuauAllocator>(
            ud: *mut ffi::c_void,
            ptr: *mut ffi::c_void,
            osize: usize,
            nsize: usize,
        ) -> *mut ffi::c_void {
            let alloc = unsafe { ud.cast::<Alloc>().as_mut().unwrap_unchecked() };
            let ptr = ptr.cast::<u8>();

            if nsize == 0 {
                if !ptr.is_null() {
                    alloc.dealloc(ptr, osize);
                }

                std::ptr::null_mut()
            } else if ptr.is_null() {
                alloc.alloc(nsize).cast()
            } else {
                alloc.realloc(ptr, osize, nsize).cast()
            }
        }

        extern "C-unwind" fn userthread<C: Config>(
            parent: *mut sys::lua_State,
            thread: *mut sys::lua_State,
        ) {
            if !parent.is_null() {
                unsafe {
                    let parent = Thread::<C>(NonNull::new_unchecked(parent), PhantomData);
                    let thread = Thread::<C>(NonNull::new_unchecked(thread), PhantomData);

                    let data = C::ThreadData::new(&parent, &thread);
                    let data = Box::into_raw(Box::new(RefCell::new(data)));

                    sys::lua_setthreaddata(thread.as_ptr(), data.cast());
                }
            } else {
                unsafe {
                    sys::lua_getthreaddata(thread)
                        .cast::<RefCell<C::ThreadData>>()
                        .drop_in_place();
                }
            }
        }

        let ptr = unsafe { sys::lua_newstate(alloc_fn::<C::Allocator>, alloc.as_ptr().cast()) };

        unsafe {
            let callbacks = sys::lua_callbacks(ptr);
            (*callbacks).userthread = Some(userthread::<C>);
        }

        Self {
            libraries: Vec::new(),
            alloc,
            main,
            ptr: NonNull::new(ptr).unwrap(),
        }
    }

    pub fn as_ptr(&self) -> *mut sys::lua_State {
        self.ptr.as_ptr()
    }

    pub fn thread(&self) -> ThreadMain<C> {
        unsafe { std::mem::transmute(self.ptr) }
    }

    pub fn stack(&self) -> &Stack<C> {
        unsafe { std::mem::transmute(&self.ptr) }
    }

    pub fn data(&self) -> &RefCell<C::MainData> {
        unsafe { self.main.as_ref() }
    }

    pub fn open_library(&mut self, name: &'static str, library: Library<C>) {
        let stack = self.stack();
        stack.reserve(3);

        stack.push_string(name);
        library.push(stack);

        stack.table_set_raw(sys::LUA_GLOBALSINDEX);
        stack.pop(1);

        self.libraries.push((name, library))
    }

    pub fn open_userdata<U: Userdata>(&self) {
        extern "C-unwind" fn dtor<U: Userdata>(_: *mut sys::lua_State, ud: *mut ffi::c_void) {
            unsafe { ud.cast::<U>().drop_in_place() };
        }

        let stack = self.stack();
        stack.push_table();

        stack.push_string(U::name());
        stack.table_set_raw_field(-2, c"__type");

        unsafe {
            sys::lua_setuserdatametatable(self.as_ptr(), U::tag() as _);
            sys::lua_setuserdatadtor(self.as_ptr(), U::tag() as _, Some(dtor::<U>));
        }
    }

    pub fn open_std(&self) {
        unsafe {
            sys::luaL_openlibs(self.as_ptr());
        }
    }

    pub fn open_base(&self) {
        unsafe { sys::luaopen_base(self.as_ptr()) };
    }

    pub fn open_coroutine(&self) {
        unsafe { sys::luaopen_coroutine(self.as_ptr()) };
    }

    pub fn open_table(&self) {
        unsafe { sys::luaopen_table(self.as_ptr()) };
    }

    pub fn open_os(&self) {
        unsafe { sys::luaopen_os(self.as_ptr()) };
    }

    pub fn open_string(&self) {
        unsafe { sys::luaopen_string(self.as_ptr()) };
    }

    pub fn open_bit(&self) {
        unsafe { sys::luaopen_bit32(self.as_ptr()) };
    }

    pub fn open_buffer(&self) {
        unsafe { sys::luaopen_buffer(self.as_ptr()) };
    }

    pub fn open_utf8(&self) {
        unsafe { sys::luaopen_utf8(self.as_ptr()) };
    }

    pub fn open_math(&self) {
        unsafe { sys::luaopen_math(self.as_ptr()) };
    }

    pub fn open_debug(&self) {
        unsafe { sys::luaopen_debug(self.as_ptr()) };
    }

    pub fn open_vector(&self) {
        unsafe { sys::luaopen_vector(self.as_ptr()) };
    }

    pub fn sandbox(&self) {
        unsafe { sys::luaL_sandbox(self.as_ptr()) }
    }

    pub fn new_thread(&self) -> ThreadRef<C> {
        let thread = self.stack().push_thread_new();
        self.stack().pop(1);

        thread
    }
}
