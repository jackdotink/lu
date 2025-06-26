use std::{cell::RefCell, ffi, marker::PhantomData, ptr::NonNull};

use crate::{
    DefaultAllocator, Library, LuauAllocator, Stack, Thread, ThreadMain, ThreadRef, Userdata,
};

pub struct State<MainData, ThreadData, Alloc: LuauAllocator = DefaultAllocator> {
    alloc: NonNull<Alloc>,
    main: NonNull<RefCell<MainData>>,
    data: PhantomData<ThreadData>,
    ptr: NonNull<sys::lua_State>,
}

impl<MainData, ThreadData, Alloc: LuauAllocator> Drop for State<MainData, ThreadData, Alloc> {
    fn drop(&mut self) {
        unsafe {
            sys::lua_close(self.ptr.as_ptr());
            self.main.drop_in_place();
            self.alloc.drop_in_place();
        }
    }
}

impl<MainData, ThreadData, Alloc: LuauAllocator> State<MainData, ThreadData, Alloc> {
    pub fn new(
        main_data: MainData,
        alloc: Alloc,
        thread_ctor: fn(
            parent: Thread<MainData, ThreadData>,
            thread: Thread<MainData, ThreadData>,
        ) -> ThreadData,
    ) -> Self {
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

            if osize == 0 {
                alloc.alloc(nsize) as _
            } else if nsize == 0 {
                alloc.dealloc(ptr, osize);
                std::ptr::null_mut()
            } else {
                alloc.realloc(ptr, osize, nsize) as _
            }
        }

        extern "C-unwind" fn userthread<MainData, ThreadData>(
            parent_ptr: *mut sys::lua_State,
            thread_ptr: *mut sys::lua_State,
        ) {
            if !parent_ptr.is_null() {
                let parent = Thread::<MainData, ThreadData>(
                    unsafe { NonNull::new_unchecked(parent_ptr) },
                    PhantomData,
                );

                let thread = Thread::<MainData, ThreadData>(
                    unsafe { NonNull::new_unchecked(thread_ptr) },
                    PhantomData,
                );

                unsafe {
                    let callbacks = sys::lua_callbacks(parent_ptr);
                    let ctor = (*callbacks).userdata.cast::<fn(
                        parent: Thread<MainData, ThreadData>,
                        thread: Thread<MainData, ThreadData>,
                    ) -> ThreadData>();

                    let data = (*ctor)(parent, thread);
                    let data = Box::into_raw(Box::new(RefCell::new(data)));

                    sys::lua_setthreaddata(thread_ptr, data.cast());
                }
            } else {
                unsafe {
                    sys::lua_getthreaddata(thread_ptr)
                        .cast::<RefCell<ThreadData>>()
                        .drop_in_place();
                }
            }
        }

        let ptr = unsafe { sys::lua_newstate(alloc_fn::<Alloc>, alloc.as_ptr().cast()) };

        unsafe {
            let callbacks = sys::lua_callbacks(ptr);
            (*callbacks).userthread = Some(userthread::<MainData, ThreadData>);
            (*callbacks).userdata = thread_ctor as *mut _;
        }

        Self {
            alloc,
            main,
            data: PhantomData,
            ptr: NonNull::new(ptr).unwrap(),
        }
    }

    pub fn as_ptr(&self) -> *mut sys::lua_State {
        self.ptr.as_ptr()
    }

    pub fn thread(&self) -> ThreadMain<MainData, ThreadData> {
        unsafe { std::mem::transmute(self.ptr) }
    }

    pub fn stack(&self) -> &Stack<MainData, ThreadData> {
        unsafe { std::mem::transmute(&self.ptr) }
    }

    pub fn data(&self) -> &RefCell<MainData> {
        unsafe { self.main.as_ref() }
    }

    pub fn open_library<L: Library<MainData, ThreadData>>(&self) {
        L::open(&self.thread())
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

    pub fn new_thread(&self) -> ThreadRef<MainData, ThreadData> {
        let thread = self.stack().push_thread_new();
        self.stack().pop(1);

        thread
    }
}
