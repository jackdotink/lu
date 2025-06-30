use std::{cell::RefCell, marker::PhantomData, ops::Deref, ptr::NonNull};

use crate::{Config, Ref, Stack, Status};

#[repr(transparent)]
pub struct Thread<C: Config>(
    pub(crate) NonNull<sys::lua_State>,
    pub(crate) PhantomData<C>,
);

impl<C: Config> Thread<C> {
    pub fn as_ptr(&self) -> *mut sys::lua_State {
        self.0.as_ptr()
    }

    pub fn main(&self) -> ThreadMain<C> {
        let ptr = unsafe { NonNull::new_unchecked(sys::lua_mainthread(self.as_ptr())) };

        ThreadMain {
            thread: Thread(ptr, PhantomData),
        }
    }

    pub fn stack(&self) -> &Stack<C> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn data(&self) -> &RefCell<C::ThreadData> {
        #[cfg(debug_assertions)]
        if self.main().as_ptr() == self.as_ptr() {
            panic!("cannot access thread data from the main thread");
        }

        unsafe {
            sys::lua_getthreaddata(self.as_ptr())
                .cast::<RefCell<C::ThreadData>>()
                .as_ref()
                .unwrap_unchecked()
        }
    }

    pub fn sandbox(&self) {
        unsafe { sys::luaL_sandboxthread(self.as_ptr()) }
    }

    pub fn resume(&self, from: Option<&Thread<C>>, nargs: u32) -> Status {
        let from = match from {
            Some(thread) => thread.as_ptr(),
            None => std::ptr::null_mut(),
        };

        unsafe { sys::lua_resume(self.as_ptr(), from, nargs as _) }.into()
    }
}

pub struct ThreadMain<C: Config> {
    pub(crate) thread: Thread<C>,
}

impl<C: Config> Deref for ThreadMain<C> {
    type Target = Thread<C>;

    fn deref(&self) -> &Self::Target {
        &self.thread
    }
}

impl<C: Config> ThreadMain<C> {
    pub fn data(&self) -> &RefCell<C::MainData> {
        unsafe {
            sys::lua_getthreaddata(self.as_ptr())
                .cast::<RefCell<C::MainData>>()
                .as_ref()
                .unwrap_unchecked()
        }
    }
}

pub struct ThreadRef<C: Config> {
    pub(crate) thread: Thread<C>,
    pub(crate) _thref: Ref<C>,
}

impl<C: Config> Deref for ThreadRef<C> {
    type Target = Thread<C>;

    fn deref(&self) -> &Self::Target {
        &self.thread
    }
}
