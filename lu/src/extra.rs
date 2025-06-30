use std::ffi::CString;

use crate::{Config, Context, FnReturn, Thread, ThreadMain};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok = sys::LUA_OK as _,
    Yield = sys::LUA_YIELD as _,
    ErrorRuntime = sys::LUA_ERRRUN as _,
    ErrorMemory = sys::LUA_ERRMEM as _,
    ErrorHander = sys::LUA_ERRERR as _,
    Break = sys::LUA_BREAK as _,
}

impl From<sys::lua_Status> for Status {
    fn from(value: sys::lua_Status) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    None = sys::LUA_TNONE as _,
    Nil = sys::LUA_TNIL as _,
    Boolean = sys::LUA_TBOOLEAN as _,
    LightUserdata = sys::LUA_TLIGHTUSERDATA as _,
    Number = sys::LUA_TNUMBER as _,
    Vector = sys::LUA_TVECTOR as _,
    String = sys::LUA_TSTRING as _,
    Table = sys::LUA_TTABLE as _,
    Function = sys::LUA_TFUNCTION as _,
    Userdata = sys::LUA_TUSERDATA as _,
    Thread = sys::LUA_TTHREAD as _,
    Buffer = sys::LUA_TBUFFER as _,
}

impl From<sys::lua_Type> for Type {
    fn from(value: sys::lua_Type) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

pub struct Ref<C: Config>(pub(crate) ThreadMain<C>, pub(crate) u32);

impl<C: Config> Drop for Ref<C> {
    fn drop(&mut self) {
        unsafe { sys::lua_unref(self.0.as_ptr(), self.1 as _) }
    }
}

pub enum Function<C: Config> {
    Normal {
        name: &'static str,
        func: extern "C-unwind" fn(ctx: Context<C>) -> FnReturn,
    },

    Continuation {
        name: &'static str,
        func: extern "C-unwind" fn(ctx: Context<C>) -> FnReturn,
        cont: extern "C-unwind" fn(ctx: Context<C>, status: Status) -> FnReturn,
    },
}

impl<C: Config> Function<C> {
    pub fn norm(
        name: &'static str,
        func: extern "C-unwind" fn(ctx: Context<C>) -> FnReturn,
    ) -> Self {
        Self::Normal { name, func }
    }

    pub fn cont(
        name: &'static str,
        func: extern "C-unwind" fn(ctx: Context<C>) -> FnReturn,
        cont: extern "C-unwind" fn(ctx: Context<C>, status: Status) -> FnReturn,
    ) -> Self {
        Self::Continuation { name, func, cont }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal { name, .. } => name,
            Self::Continuation { name, .. } => name,
        }
    }
}
