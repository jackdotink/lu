use std::ffi::CString;

use crate::{Stack, ThreadMain};

pub trait Library<MainData, ThreadData> {
    fn name() -> &'static str;
    fn push(stack: &Stack<MainData, ThreadData>);
    fn open(thread: &ThreadMain<MainData, ThreadData>) {
        Self::push(thread.stack());

        let name = CString::new(Self::name()).expect("library name has null bytes");
        unsafe { sys::lua_setglobal(thread.as_ptr(), name.as_ptr()) }
    }
}
