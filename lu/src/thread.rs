use std::{cell::RefCell, marker::PhantomData, ops::Deref, ptr::NonNull};

use crate::{Ref, Stack, Status, ThreadData};

#[repr(transparent)]
pub struct Thread<MD, TD: ThreadData<MD>>(
    pub(crate) NonNull<sys::lua_State>,
    pub(crate) PhantomData<(MD, TD)>,
);

impl<MD, TD: ThreadData<MD>> Thread<MD, TD> {
    pub fn as_ptr(&self) -> *mut sys::lua_State {
        self.0.as_ptr()
    }

    pub fn main(&self) -> ThreadMain<MD, TD> {
        let ptr = unsafe { NonNull::new_unchecked(sys::lua_mainthread(self.as_ptr())) };

        ThreadMain {
            thread: Thread(ptr, PhantomData),
        }
    }

    pub fn stack(&self) -> &Stack<MD, TD> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn data(&self) -> &RefCell<TD> {
        unsafe {
            sys::lua_getthreaddata(self.as_ptr())
                .cast::<RefCell<TD>>()
                .as_ref()
                .unwrap_unchecked()
        }
    }

    pub fn sandbox(&self) {
        unsafe { sys::luaL_sandboxthread(self.as_ptr()) }
    }

    pub fn resume(&self, from: Option<&Thread<MD, TD>>, nargs: u32) -> Status {
        let from = match from {
            Some(thread) => thread.as_ptr(),
            None => std::ptr::null_mut(),
        };

        unsafe { sys::lua_resume(self.as_ptr(), from, nargs as _) }.into()
    }
}

pub struct ThreadMain<MD, TD: ThreadData<MD>> {
    pub(crate) thread: Thread<MD, TD>,
}

impl<MD, TD: ThreadData<MD>> Deref for ThreadMain<MD, TD> {
    type Target = Thread<MD, TD>;

    fn deref(&self) -> &Self::Target {
        &self.thread
    }
}

impl<MD, TD: ThreadData<MD>> ThreadMain<MD, TD> {
    pub fn data(&self) -> &RefCell<MD> {
        unsafe {
            sys::lua_getthreaddata(self.as_ptr())
                .cast::<RefCell<MD>>()
                .as_ref()
                .unwrap_unchecked()
        }
    }
}

pub struct ThreadRef<MD, TD: ThreadData<MD>> {
    pub(crate) thread: Thread<MD, TD>,
    pub(crate) _thref: Ref<MD, TD>,
}

impl<MD, TD: ThreadData<MD>> Deref for ThreadRef<MD, TD> {
    type Target = Thread<MD, TD>;

    fn deref(&self) -> &Self::Target {
        &self.thread
    }
}
