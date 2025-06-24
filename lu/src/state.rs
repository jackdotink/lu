use std::{cell::RefCell, ffi, marker::PhantomData, ptr::NonNull};

use crate::{
    alloc::{DefaultAllocator, LuauAllocator}, Library, Stack, Thread,
    ThreadMain,
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
        unsafe { std::mem::transmute(self.ptr) }
    }

    pub fn data(&self) -> &RefCell<MainData> {
        unsafe { self.main.as_ref() }
    }

    pub fn open<L: Library<MainData, ThreadData>>(&self) {
        L::open(&self.thread())
    }

    pub fn sandbox(&self) {
        unsafe { sys::luaL_sandbox(self.as_ptr()) }
    }
}
