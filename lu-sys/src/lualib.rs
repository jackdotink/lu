#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{
    ffi::{CStr, c_char, c_double, c_float, c_int, c_uint, c_void},
    ptr::null_mut,
};

use super::*;

pub unsafe fn luaL_typeerror(L: *mut lua_State, narg: c_int, tname: *const c_char) -> ! {
    unsafe { luaL_typeerrorL(L, narg, tname) }
}

pub unsafe fn luaL_argerror(L: *mut lua_State, narg: c_int, extramsg: *const c_char) -> ! {
    unsafe { luaL_argerrorL(L, narg, extramsg) }
}

#[repr(C)]
pub struct luaL_Reg {
    pub name: *const c_char,
    pub func: lua_CFunction,
}

unsafe extern "C-unwind" {
    pub fn luaL_register(L: *mut lua_State, libname: *const c_char, l: *const luaL_Reg);

    pub fn luaL_getmetafield(L: *mut lua_State, obj: c_int, e: *const c_char) -> c_int;

    pub fn luaL_callmeta(L: *mut lua_State, obj: c_int, e: *const c_char) -> c_int;

    pub fn luaL_typeerrorL(L: *mut lua_State, narg: c_int, tname: *const c_char) -> !;

    pub fn luaL_argerrorL(L: *mut lua_State, narg: c_int, extramsg: *const c_char) -> !;

    pub fn luaL_checklstring(L: *mut lua_State, numArg: c_int, l: *mut usize) -> *const c_char;

    pub fn luaL_optlstring(
        L: *mut lua_State,
        numArg: c_int,
        def: *const c_char,
        l: *mut usize,
    ) -> *const c_char;

    pub fn luaL_checknumber(L: *mut lua_State, numArg: c_int) -> c_double;

    pub fn luaL_optnumber(L: *mut lua_State, nArg: c_int, def: c_double) -> c_double;

    pub fn luaL_checkboolean(L: *mut lua_State, narg: c_int) -> c_int;

    pub fn luaL_optboolean(L: *mut lua_State, narg: c_int, def: c_int) -> c_int;

    pub fn luaL_checkinteger(L: *mut lua_State, numArg: c_int) -> c_int;

    pub fn luaL_optinteger(L: *mut lua_State, nArg: c_int, def: c_int) -> c_int;

    pub fn luaL_checkunsigned(L: *mut lua_State, numArg: c_int) -> c_uint;

    pub fn luaL_optunsigned(L: *mut lua_State, numArg: c_int, def: c_uint) -> c_uint;

    pub fn luaL_checkvector(L: *mut lua_State, narg: c_int) -> *const c_float;

    pub fn luaL_optvector(L: *mut lua_State, narg: c_int, def: *const c_float) -> *const c_float;

    pub fn luaL_checkstack(L: *mut lua_State, sz: c_int, msg: *const c_char);

    pub fn luaL_checktype(L: *mut lua_State, narg: c_int, t: c_int);

    pub fn luaL_checkany(L: *mut lua_State, narg: c_int);

    pub fn luaL_newmetatable(L: *mut lua_State, tname: *const c_char) -> c_int;

    pub fn luaL_checkudata(L: *mut lua_State, ud: c_int, tname: *const c_char) -> *mut c_void;

    pub fn luaL_checkbuffer(L: *mut lua_State, narg: c_int, len: *mut usize) -> *mut c_void;

    pub fn luaL_where(L: *mut lua_State, lvl: c_int);

    pub fn luaL_checkoption(
        L: *mut lua_State,
        narg: c_int,
        def: *const c_char,
        lst: *const *const c_char,
    ) -> c_int;

    pub fn luaL_tolstring(L: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char;

    pub fn luaL_newstate() -> *mut lua_State;

    pub fn luaL_findtable(
        L: *mut lua_State,
        idx: c_int,
        fname: *const c_char,
        szhint: c_int,
    ) -> *const c_char;

    pub fn luaL_typename(L: *mut lua_State, idx: c_int) -> *const c_char;

    pub fn luaL_callyieldable(L: *mut lua_State, nargs: c_int, nresults: c_int) -> c_int;
}

pub unsafe fn luaL_argcheck(L: *mut lua_State, cond: bool, arg: c_int, extramsg: *const c_char) {
    if !cond {
        unsafe { luaL_argerror(L, arg, extramsg) }
    }
}

pub unsafe fn luaL_argexpected(L: *mut lua_State, cond: bool, arg: c_int, tname: *const c_char) {
    if !cond {
        unsafe { luaL_typeerror(L, arg, tname) }
    }
}

pub unsafe fn luaL_checkstring(L: *mut lua_State, n: c_int) -> *const c_char {
    unsafe { luaL_checklstring(L, n, null_mut()) }
}

pub unsafe fn luaL_optstring(L: *mut lua_State, n: c_int, d: *const c_char) -> *const c_char {
    unsafe { luaL_optlstring(L, n, d, null_mut()) }
}

pub unsafe fn luaL_getmetatable(L: *mut lua_State, n: *const c_char) -> lua_Type {
    unsafe { lua_getfield(L, LUA_REGISTRYINDEX, n) }
}

pub unsafe fn luaL_opt<T>(
    L: *mut lua_State,
    f: fn(L: *mut lua_State, n: c_int) -> T,
    n: c_int,
    d: T,
) -> T {
    unsafe { if lua_isnoneornil(L, n) { d } else { f(L, n) } }
}

#[repr(C)]
pub struct luaL_Strbuf {
    pub p: *mut c_char,
    pub end: *mut c_char,
    pub L: *mut lua_State,
    pub storage: *mut c_void, // !!!
    pub buffer: [c_char; LUA_BUFFERSIZE as usize],
}

pub unsafe fn luaL_addchar(B: *mut luaL_Strbuf, c: c_char) {
    unsafe {
        if (*B).p >= (*B).end {
            luaL_prepbuffsize(B, 1);
        }

        (*B).p.write(c);
        (*B).p = (*B).p.add(1);
    }
}

pub unsafe fn luaL_addstring(B: *mut luaL_Strbuf, s: *const c_char) {
    unsafe { luaL_addlstring(B, s, libc::strlen(s)) }
}

unsafe extern "C-unwind" {
    pub fn luaL_buffinit(L: *mut lua_State, B: *mut luaL_Strbuf);

    pub fn luaL_buffinitsize(L: *mut lua_State, B: *mut luaL_Strbuf, size: usize) -> *mut c_char;

    pub fn luaL_prepbuffsize(B: *mut luaL_Strbuf, size: usize) -> *mut c_char;

    pub fn luaL_addlstring(B: *mut luaL_Strbuf, s: *const c_char, l: usize);

    pub fn luaL_addvalue(B: *mut luaL_Strbuf);

    pub fn luaL_addvalueany(B: *mut luaL_Strbuf, idx: c_int);

    pub fn luaL_pushresult(B: *mut luaL_Strbuf);

    pub fn luaL_pushresultsize(B: *mut luaL_Strbuf, size: usize);
}

pub const LUA_COLIBNAME: &CStr = c"coroutine";

pub const LUA_TABLIBNAME: &CStr = c"table";

pub const LUA_OSLIBNAME: &CStr = c"os";

pub const LUA_STRLIBNAME: &CStr = c"string";

pub const LUA_BITLIBNAME: &CStr = c"bit32";

pub const LUA_BUFFERLIBNAME: &CStr = c"buffer";

pub const LUA_UTF8LIBNAME: &CStr = c"utf8";

pub const LUA_MATHLIBNAME: &CStr = c"math";

pub const LUA_DBLIBNAME: &CStr = c"debug";

pub const LUA_VECLIBNAME: &CStr = c"vector";

unsafe extern "C-unwind" {
    pub fn luaopen_base(L: *mut lua_State) -> c_int;

    pub fn luaopen_coroutine(L: *mut lua_State) -> c_int;

    pub fn luaopen_table(L: *mut lua_State) -> c_int;

    pub fn luaopen_os(L: *mut lua_State) -> c_int;

    pub fn luaopen_string(L: *mut lua_State) -> c_int;

    pub fn luaopen_bit32(L: *mut lua_State) -> c_int;

    pub fn luaopen_buffer(L: *mut lua_State) -> c_int;

    pub fn luaopen_utf8(L: *mut lua_State) -> c_int;

    pub fn luaopen_math(L: *mut lua_State) -> c_int;

    pub fn luaopen_debug(L: *mut lua_State) -> c_int;

    pub fn luaopen_vector(L: *mut lua_State) -> c_int;

    pub fn luaL_openlibs(L: *mut lua_State);

    pub fn luaL_sandbox(L: *mut lua_State);

    pub fn luaL_sandboxthread(L: *mut lua_State);
}
