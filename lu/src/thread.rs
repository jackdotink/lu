use std::{cell::RefCell, marker::PhantomData, ops::Deref, ptr::NonNull};

use crate::{Ref, Stack};

#[repr(transparent)]
pub struct Thread<MainData, ThreadData>(
    pub(crate) NonNull<sys::lua_State>,
    pub(crate) PhantomData<(MainData, ThreadData)>,
);

impl<MainData, ThreadData> Thread<MainData, ThreadData> {
    pub fn as_ptr(&self) -> *mut sys::lua_State {
        self.0.as_ptr()
    }

    pub fn main(&self) -> ThreadMain<MainData, ThreadData> {
        let ptr = unsafe { NonNull::new_unchecked(sys::lua_mainthread(self.as_ptr())) };

        ThreadMain {
            thread: Thread(ptr, PhantomData),
        }
    }

    pub fn stack(&self) -> &Stack<MainData, ThreadData> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn data(&self) -> &RefCell<ThreadData> {
        unsafe {
            sys::lua_getthreaddata(self.as_ptr())
                .cast::<RefCell<ThreadData>>()
                .as_ref()
                .unwrap_unchecked()
        }
    }

    pub fn sandbox(&self) {
        unsafe { sys::luaL_sandboxthread(self.as_ptr()) }
    }
}

pub struct ThreadMain<MainData, ThreadData> {
    pub(crate) thread: Thread<MainData, ThreadData>,
}

impl<MainData, ThreadData> Deref for ThreadMain<MainData, ThreadData> {
    type Target = Thread<MainData, ThreadData>;

    fn deref(&self) -> &Self::Target {
        &self.thread
    }
}

impl<MainData, ThreadData> ThreadMain<MainData, ThreadData> {
    pub fn data(&self) -> &RefCell<MainData> {
        unsafe {
            sys::lua_getthreaddata(self.as_ptr())
                .cast::<RefCell<MainData>>()
                .as_ref()
                .unwrap_unchecked()
        }
    }
}

pub struct ThreadRef<MainData, ThreadData> {
    pub(crate) thread: Thread<MainData, ThreadData>,
    pub(crate) _thref: Ref<MainData, ThreadData>,
}

impl<MainData, ThreadData> Deref for ThreadRef<MainData, ThreadData> {
    type Target = Thread<MainData, ThreadData>;

    fn deref(&self) -> &Self::Target {
        &self.thread
    }
}
