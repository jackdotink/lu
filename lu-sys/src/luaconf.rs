use std::ffi::c_int;

pub const LUA_IDSIZE: c_int = 256;
pub const LUA_MINSTACK: c_int = 20;
pub const LUAI_MAXCSTACK: c_int = 8000;
pub const LUAI_MAXCALLS: c_int = 20000;
pub const LUAI_MAXCCALLS: c_int = 200;
pub const LUA_BUFFERSIZE: c_int = 512;
pub const LUA_UTAG_LIMIT: c_int = 128;
pub const LUA_LUTAG_LIMIT: c_int = 128;
pub const LUA_SIZECLASSES: c_int = 40;
pub const LUA_MEMORY_CATEGORIES: c_int = 256;
pub const LUA_MINSTRTABSIZE: c_int = 32;
pub const LUA_MAXCAPTURES: c_int = 32;
pub const LUA_VECTOR_SIZE: c_int = 3;
