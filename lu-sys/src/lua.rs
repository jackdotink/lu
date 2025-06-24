#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::marker::{PhantomData, PhantomPinned};
use std::{
    ffi::{c_char, c_double, c_float, c_int, c_uchar, c_uint, c_void},
    ptr::null_mut,
};

use super::*;

/// Indicates that a call should return all values to the caller.
///
/// This is used with [`lua_call`] and [`lua_pcall`] to specify that all returned
/// values should be pushed onto the stack.
pub const LUA_MULTRET: c_int = -1;

/// The pseudo-index for the registry.
///
/// The registry is a pre-defined table that is only accessible from the C API.
/// Integer keys in this table should not be used, as they are reserved for
/// Luau's ref system.
///
/// Care should be taken that keys do not conflict. A great way to avoid this is
/// to use a light userdata key pointing to a static variable, as this will
/// always be unique.
pub const LUA_REGISTRYINDEX: c_int = -LUAI_MAXCSTACK - 2000;

/// The pseudo-index for the environment table of the running function.
pub const LUA_ENVIRONINDEX: c_int = -LUAI_MAXCSTACK - 2001;

/// The pseudo-index for the global environment table.
pub const LUA_GLOBALSINDEX: c_int = -LUAI_MAXCSTACK - 2002;

/// Produces the pseudo-indicies for upvalues of C functions.
///
/// A C function that captures upvalues can get a pseudo-index for each upvalue
/// using this function. The first upvalue is at `lua_upvalueindex(1)`, and the
/// n-th upvalue is at `lua_upvalueindex(n)`.
///
/// This function will produce an acceptable but invalid pseudo-index for any
/// input.
pub fn lua_upvalueindex(i: c_int) -> c_int {
    LUA_GLOBALSINDEX - i
}

/// Checks if the given index is a pseudo-index.
///
/// This function checks if the given index is a pseudo-index, which includes
/// the registry, environment, and global environment indices.
pub fn lua_ispseudo(i: c_int) -> bool {
    i <= LUA_REGISTRYINDEX
}

/// The status codes returned by the Luau VM.
///
/// These codes indicate the result of a function call or operation in the Luau
/// VM. As an example [`lua_pcall`] might return [`LUA_OK`] if the call was
/// successful, or [`LUA_YIELD`] if the call yielded execution, or
/// [`LUA_ERRRUN`] if there was a runtime error.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum lua_Status {
    /// The call was successful and the function returned normally.
    LUA_OK = 0,

    /// The call yielded execution using either [`lua_yield`] or `coroutine.yield`.
    LUA_YIELD,

    /// The call encountered a runtime error.
    LUA_ERRRUN,

    /// This variant exists only for backwards compatibility. It is not used.
    LUA_ERRSYNTAX,

    /// The call encountered a memory allocation error.
    LUA_ERRMEM,

    /// The call encountered an error while running the error handler.
    LUA_ERRERR,

    /// The call was interrupted by a break.
    LUA_BREAK,
}

pub use lua_Status::*;

/// The status codes of a coroutine.
///
/// This is only returned by [`lua_costatus`].
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum lua_CoStatus {
    LUA_CORUN = 0,
    LUA_COSUS,
    LUA_CONOR,
    LUA_COFIN,
    LUA_COERR,
}

pub use lua_CoStatus::*;

/// Represents a Luau thread, stack, and environment. The main structure for
/// interacting with the Luau VM.
///
/// This structure is opaque and should never exist, except as a pointer.
#[repr(C)]
pub struct lua_State {
    _data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// The type of a C function that can be called from Luau.
///
/// This function takes a pointer to a [`lua_State`] and returns the number of
/// results it pushes onto the stack. It can also yield execution, in which case
/// it should return the result from [`lua_yield`].
pub type lua_CFunction = extern "C-unwind" fn(L: *mut lua_State) -> c_int;

/// The type of a continuation function that can be called from Luau.
///
/// When a C function yields, either by yielding directly or by calling a
/// function that yields, it's continuation will be run when the coroutine is
/// resumed.
///
/// The following Luau and C code segments are equivalent:
///
/// ```lua
/// local function foo()
///     print("foo")
///     coroutine.yield()
///     print("bar")
/// end
/// ```
///
/// ```c
/// int foo(lua_State* L) {
///     printf("foo\n");
///     return lua_yield(L, 0);
/// }
///
/// int foo_cont(lua_State* L, int status) {
///     printf("bar\n");
///     return 0;
/// }
/// ```
pub type lua_Continuation = extern "C-unwind" fn(L: *mut lua_State, status: lua_Status) -> c_int;

/// The type of the memory allocator function used by Luau's VM.
///
/// * `ud` is the same userdata pointer as was passed to [`lua_newstate`].
/// * `ptr` is the pointer to the memory block to be reallocated, or null if
///   osize is zero.
/// * `osize` is the size of the memory block pointed to by `ptr`.
/// * `nsize` is the new size of the memory block to be allocated.
///
/// If `osize` is zero, `ptr` will be null and the function should allocate a
/// new memory block of size `nsize`.
///
/// If `nsize` is zero, the function should free the memory block pointed to be
/// `ptr`.
///
/// If `osize` and `nsize` are both non-zero, the function should reallocate the
/// memory block pointed to by `ptr` to the new size `nsize`.
///
/// # Example
///
/// ```
/// extern "C-unwind" fn alloc(
///     ud: *mut c_void,
///     ptr: *mut c_void,
///     osize: usize,
///     nsize: usize
/// ) -> *mut c_void {
///     if nsize == 0 {
///         unsafe { libc::free(ptr) };
///         std::ptr::null_mut()    
///     } else {
///         unsafe { libc::realloc(ptr, nsize) }
///     }
/// }
/// ```
pub type lua_Alloc = extern "C-unwind" fn(
    ud: *mut c_void,
    ptr: *mut c_void,
    osize: usize,
    nsize: usize,
) -> *mut c_void;

/// The different types of Luau values.
///
/// This is returned by [`lua_type`] and is taken as an argument by
/// [`lua_typename`] and [`luaL_checktype`].
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum lua_Type {
    LUA_TNONE = -1,
    LUA_TNIL = 0,
    LUA_TBOOLEAN = 1,

    LUA_TLIGHTUSERDATA,
    LUA_TNUMBER,
    LUA_TVECTOR,

    LUA_TSTRING,

    LUA_TTABLE,
    LUA_TFUNCTION,
    LUA_TUSERDATA,
    LUA_TTHREAD,
    LUA_TBUFFER,
}

pub use lua_Type::*;

/// The type of Luau numbers.
pub type lua_Number = c_double;

/// The type of Luau integers.
pub type lua_Integer = c_int;

/// The type of Luau unsigned integers.
pub type lua_Unsigned = c_uint;

unsafe extern "C-unwind" {
    /// Creates a new [`lua_State`].
    ///
    /// This function takes an allocator function and a userdata pointer.
    pub fn lua_newstate(f: lua_Alloc, ud: *mut c_void) -> *mut lua_State;

    /// Closes the passed [`lua_State`].
    ///
    /// This function will free all memory associated with the state and
    /// invalidate the pointer.
    pub fn lua_close(L: *mut lua_State);

    /// Creates a new thread.
    ///
    /// This function creates a new thread, pushes it onto the stack, and
    /// returns the pointer to the new thread. Threads, like all other Luau
    /// objects, are subject to garbage collection.
    pub fn lua_newthread(L: *mut lua_State) -> *mut lua_State;

    /// Returns the main thread of the given state.
    ///
    /// This function returns the main thread of the given state, or the pointer
    /// initially returned by [`lua_newstate`].
    pub fn lua_mainthread(L: *mut lua_State) -> *mut lua_State;

    /// TODO: Document this function.
    pub fn lua_resetthread(L: *mut lua_State);

    /// TODO: Document this function.
    pub fn lua_isthreadreset(L: *mut lua_State) -> c_int;

    /// Transforms a stack index into an index that is not relative to the top
    /// of the stack.
    ///
    /// Passing psuedo-indices or positive indices will return the same
    /// index, while negative indices will be transformed to a positive index.
    pub fn lua_absindex(L: *mut lua_State, idx: c_int) -> c_int;

    /// Returns the number of items on the stack.
    ///
    /// This function returns the top index of the stack, or the number of items
    /// on the stack. A return of `0` indicates that the stack is empty.
    pub fn lua_gettop(L: *mut lua_State) -> c_int;

    /// Sets the top of the stack to the given index.
    ///
    /// Accepts any index, including `0`. If the index is greater than the
    /// current top, the stack will be extended with `nil` values.
    pub fn lua_settop(L: *mut lua_State, idx: c_int);

    /// Pushes a value onto the stack.
    ///
    /// This function copies the value at the given index, and pushes the copy
    /// onto the stack.
    pub fn lua_pushvalue(L: *mut lua_State, idx: c_int);

    /// Removes the value at the given index from the stack.
    ///
    /// This function removes the value at the given index from the stack,
    /// shifting all values above it down by one.
    pub fn lua_remove(L: *mut lua_State, idx: c_int);

    /// Moves the value at the given index to the top of the stack.
    ///
    /// This function moves the value at the given index to the top of the
    /// stack, shifting all values above it down by one.
    pub fn lua_insert(L: *mut lua_State, idx: c_int);

    /// Replaces the given index with the value at the top of the stack.
    ///
    /// This function replaces the value at the given index with the value at
    /// the top of the stack, and then pops the top value off the stack.
    pub fn lua_replace(L: *mut lua_State, idx: c_int);

    /// Attempts to make space on the stack for at least `n` more values.
    ///
    /// This function will attempt to make space on the stack for at least `sz`
    /// more values. If the stack is already large enough, this function does
    /// nothing. If the stack is not large enough, it will attempt to grow the
    /// stack to accommodate the new values.
    ///
    /// If the stack cannot be grown, this function will return `0`, otherwise
    /// it will return `1`.
    pub fn lua_checkstack(L: *mut lua_State, sz: c_int) -> c_int;

    /// TODO: Document this function.
    pub fn lua_rawcheckstack(L: *mut lua_State, sz: c_int);

    /// Moves values from the top of one stack to another stack.
    ///
    /// This function moves `n` values from the top of the `from` stack to the
    /// top of the `to` stack. This can be thought of as a "slide" between the
    /// two stacks.
    pub fn lua_xmove(from: *mut lua_State, to: *mut lua_State, n: c_int);

    /// Pushes a value from an index of one stack onto another stack.
    ///
    /// This function pushes a copy of the value at the given index of the
    /// `from` stack onto the top of the `to` stack.
    pub fn lua_xpush(from: *mut lua_State, to: *mut lua_State, idx: c_int);

    /// Does the given index contain a value of type number.
    pub fn lua_isnumber(L: *mut lua_State, idx: c_int) -> c_int;

    /// Does the given index contain a value of type string.
    pub fn lua_isstring(L: *mut lua_State, idx: c_int) -> c_int;

    /// Does the given index contain a value of type function which is a C
    /// function.
    pub fn lua_iscfunction(L: *mut lua_State, idx: c_int) -> c_int;

    /// Does the given index contain a value of type function which is a Luau
    /// function.
    pub fn lua_isLfunction(L: *mut lua_State, idx: c_int) -> c_int;

    /// Does the given index contain a value of type userdata.
    pub fn lua_isuserdata(L: *mut lua_State, idx: c_int) -> c_int;

    /// Returns the type of the value at the given index.
    pub fn lua_type(L: *mut lua_State, idx: c_int) -> lua_Type;

    /// Returns the name of the given type.
    pub fn lua_typename(L: *mut lua_State, tp: lua_Type) -> *const c_char;

    /// Compares the values at the two indices for equality.
    ///
    /// This function compares the values at the two indices for equality, using
    /// Luau's equality rules (`==`).
    pub fn lua_equal(L: *mut lua_State, idx1: c_int, idx2: c_int) -> c_int;

    /// Compares the values at the two indices for raw equality.
    ///
    /// This function compares the values at the two indices for raw equality,
    /// using Luau's raw equality rules (does not hit metamethods).
    pub fn lua_rawequal(L: *mut lua_State, idx1: c_int, idx2: c_int) -> c_int;

    /// Compares the values at the two indices for less than.
    ///
    /// This function compares the values at the two indices for less than,
    /// using Luau's less than rules (`<`).
    ///
    /// Greater than comparison can be done by swapping the indices.
    pub fn lua_lessthan(L: *mut lua_State, idx1: c_int, idx2: c_int) -> c_int;

    /// Attempts to convert the value at the given index to a number.
    ///
    /// This function attempts to convert the value at the given index to a
    /// number, returning the value if successful, or `0.0` if the conversion
    /// failed.
    ///
    /// As `0.0` is a valid Luau number and cannot be used to indicate failure,
    /// the function also allows for an optional pointer which it will set to
    /// `0` if the conversion failed, or `1` if it succeeded.
    ///
    /// This function will leave numbers as is, and will transform strings which
    /// can be parsed as numbers into numbers. This will modify the stack.
    pub fn lua_tonumberx(L: *mut lua_State, idx: c_int, isnum: *mut c_int) -> lua_Number;

    /// Attempts to convert the value at the given index to an integer.
    ///
    /// This function operates similarly to [`lua_tonumberx`], but adds the step
    /// of flooring the result to an integer. While a string may be turned into
    /// a number on the stack, the value on the stack will not be floored to an
    /// integer.
    pub fn lua_tointegerx(L: *mut lua_State, idx: c_int, isnum: *mut c_int) -> lua_Integer;

    /// Attempts to convert the value at the given index to an unsigned integer.
    ///
    /// This function operates in the exact same way as [`lua_tointegerx`], but
    /// returns a [`lua_Unsigned`] instead of a [`lua_Integer`]. If the value
    /// is negative the output is undefined.
    pub fn lua_tounsignedx(L: *mut lua_State, idx: c_int, isnum: *mut c_int) -> lua_Unsigned;

    /// Returns the value of the vector at the given index.
    ///
    /// If the value at the given index is not a vector, this function will
    /// return null. Otherwise, it will return a pointer to the vector's
    /// floats.
    pub fn lua_tovector(L: *mut lua_State, idx: c_int) -> *const c_float;

    /// Returns the truthiness of the value at the given index.
    ///
    /// This function returns `0` if the value at the given index is `nil` or
    /// `false`, and `1` for all other values.
    pub fn lua_toboolean(L: *mut lua_State, idx: c_int) -> c_int;

    /// Attempts to convert the value at the given index to a string and returns
    /// its value and length.
    ///
    /// This function attempts to convert the value at the given index to a
    /// string, returning a pointer to the string if successful, or null if the
    /// conversion failed.
    ///
    /// If `len` is not null, it will be set to the length of the string if the
    /// conversion was successful.
    ///
    /// If the value is a string, it will be returned as is. If the value is a
    /// number, it will be converted into a string and the stack will be
    /// modified.
    pub fn lua_tolstring(L: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char;

    /// Returns the value of the string at the given index as well as its atom.
    ///
    /// If the value at the given index is not a string, this function will
    /// return null. Otherwise, it will return a pointer to the string. Unlike
    /// [`lua_tolstring`], this function does not do any conversions or modify
    /// the stack.
    ///
    /// If `atom` is not null, it will be set to the atom of the string.
    pub fn lua_tostringatom(L: *mut lua_State, idx: c_int, atom: *mut c_int) -> *const c_char;

    /// Returns the value of the string at the given index as well as its atom
    /// and length.
    ///
    /// If the value at the given index is not a string, this function will
    /// return null. Otherwise, it will return a pointer to the string. Unlike
    /// [`lua_tolstring`], this function does not do any conversions or modify
    /// the stack.
    ///
    /// If `atom` is not null, it will be set to the atom of the string. If
    /// `len` is not null, it will be set to the length of the string.
    pub fn lua_tolstringatom(
        L: *mut lua_State,
        idx: c_int,
        len: *mut usize,
        atom: *mut c_int,
    ) -> *const c_char;

    /// Returns the value of the namecall method string as well as its atom.
    ///
    /// If not within a namecall context, this function will return null.
    /// Otherwise, it will return a pointer to the string.
    ///
    /// If `atom` is not null, it will be set to the atom of the string.
    pub fn lua_namecallatom(L: *mut lua_State, atom: *mut c_int) -> *const c_char;

    /// Returns the length of the value at the given index.
    ///
    /// Length is defined differently for different types:
    ///
    /// * For strings, it returns the length of the string in bytes.
    /// * For userdata, it returns the size of the userdata in bytes.
    /// * For buffers, it returns the length of the buffer in bytes.
    /// * For tables, it returns the same value that the length operator (`#`)
    ///   would return.
    ///
    /// If the value is not one of these types, this function will return `0`.
    pub fn lua_objlen(L: *mut lua_State, idx: c_int) -> c_int;

    /// Returns a pointer to the C function at the given index.
    ///
    /// If the value at the given index is not a C function, this function will
    /// return null. Otherwise, it will return a pointer to the C function.
    pub fn lua_tocfunction(L: *mut lua_State, idx: c_int) -> Option<lua_CFunction>;

    /// Returns the pointer of the light userdata at the given index.
    ///
    /// If the value at the given index is not a light userdata, this function
    /// will return null. Otherwise, it will return the pointer that the light
    /// userdata contains.
    pub fn lua_tolightuserdata(L: *mut lua_State, idx: c_int) -> *mut c_void;

    /// Returns the pointer to the light userdata at the given index if it has the
    /// given tag.
    ///
    /// If the value at the given index is not a light userdata, or if it does
    /// not have the given tag, this function will return null. Otherwise,
    /// it will return the pointer that the light userdata contains.
    pub fn lua_tolightuserdatatagged(L: *mut lua_State, idx: c_int, tag: c_int) -> *mut c_void;

    /// Returns the pointer to the userdata at the given index.
    ///
    /// If the value at the given index is not a userdata, this function will
    /// return null. Otherwise, it will return a pointer to the userdata.
    pub fn lua_touserdata(L: *mut lua_State, idx: c_int) -> *mut c_void;

    /// Returns the pointer to the userdata at the given index if it has the
    /// given tag.
    ///
    /// If the value at the given index is not a userdata, or if it does not
    /// have the given tag, this function will return null. Otherwise, it will
    /// return a pointer to the userdata.
    pub fn lua_touserdatatagged(L: *mut lua_State, idx: c_int, tag: c_int) -> *mut c_void;

    /// Returns the tag of the userdata at the given index.
    ///
    /// If the value at the given index is not a userdata, this function will
    /// return `-1`. Otherwise, it will return the tag of the userdata.
    pub fn lua_userdatatag(L: *mut lua_State, idx: c_int) -> c_int;

    /// Returns the tag of the light userdata at the given index.
    ///
    /// If the value at the given index is not a light userdata, this function
    /// will return `-1`. Otherwise, it will return the tag of the light
    /// userdata.
    pub fn lua_lightuserdatatag(L: *mut lua_State, idx: c_int) -> c_int;

    /// Returns the thread at the given index.
    ///
    /// If the value at the given index is not a thread, this function will
    /// return null. Otherwise, it will return a pointer to the thread's
    /// [`lua_State`].
    pub fn lua_tothread(L: *mut lua_State, idx: c_int) -> *mut lua_State;

    /// Returns the buffer at the given index.
    ///
    /// If the value at the given index is not a buffer, this function will
    /// return null. Otherwise, it will return a pointer to the buffer's data.
    ///
    /// If `len` is not null, it will be set to the length of the buffer.
    pub fn lua_tobuffer(L: *mut lua_State, idx: c_int, len: *mut usize) -> *mut c_void;

    /// Returns the pointer of the value at the given index.
    ///
    /// The "pointer of the value" is defined differently for different types:
    ///
    /// * For userdata, it returns the pointer to the userdata.
    /// * For light userdata, it returns the pointer that the light userdata
    ///   contains.
    /// * For strings, tables, functions, threads, and buffers, it returns the
    ///   pointer to the value's data on the heap. The contents of this pointer
    ///   are undefined.
    ///
    /// For all other types, this function will return null.
    pub fn lua_topointer(L: *mut lua_State, idx: c_int) -> *const c_void;

    /// Pushes a nil value onto the top of the stack.
    pub fn lua_pushnil(L: *mut lua_State);

    /// Pushes a number onto the top of the stack.
    pub fn lua_pushnumber(L: *mut lua_State, n: lua_Number);

    /// Pushes an integer onto the top of the stack.
    pub fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);

    /// Pushes an unsigned integer onto the top of the stack.
    pub fn lua_pushunsigned(L: *mut lua_State, n: lua_Unsigned);

    /// Pushes a vector onto the top of the stack.
    pub fn lua_pushvector(L: *mut lua_State, x: c_float, y: c_float, z: c_float);

    /// Pushes a string with a length onto the top of the stack.
    pub fn lua_pushlstring(L: *mut lua_State, s: *const c_char, l: usize);

    /// Pushes a string onto the top of the stack.
    pub fn lua_pushstring(L: *mut lua_State, s: *const c_char);

    /// Pushes a C function with upvalues and continuation onto the top of the stack.
    pub fn lua_pushcclosurek(
        L: *mut lua_State,
        r#fn: lua_CFunction,
        debugname: *const c_char,
        nup: c_int,
        cont: Option<lua_Continuation>,
    );

    /// Pushes a boolean onto the top of the stack.
    pub fn lua_pushboolean(L: *mut lua_State, b: c_int);

    /// Pushes the thread onto the top of its own stack.
    ///
    /// This function will return a 1 is this thread is the main thread of its
    /// state.
    pub fn lua_pushthread(L: *mut lua_State) -> c_int;

    /// Pushes a light userdata with a tag onto the top of the stack.
    pub fn lua_pushlightuserdatatagged(L: *mut lua_State, p: *mut c_void, tag: c_int);

    /// Pushes a new userdata with a tag onto the top of the stack.
    ///
    /// This function will allocate a new userdata of the given size, and push
    /// it onto the top of the stack. It will return a pointer to the allocated
    /// userdata.
    pub fn lua_newuserdatatagged(L: *mut lua_State, size: usize, tag: c_int) -> *mut c_void;

    /// Pushes a new userdata with a tag and a metatable onto the top of the
    /// stack.
    ///
    /// This function will allocate a new userdata of the given size, push it
    /// onto the top of the stack, and set its metatable to the table registered
    /// with the given tag. It will return a pointer to the allocated userdata.
    pub fn lua_newuserdatataggedwithmetatable(
        L: *mut lua_State,
        size: usize,
        tag: c_int,
    ) -> *mut c_void;

    /// Pushes a new userdata with a destructor onto the top of the stack.
    ///
    /// This function will allocate a new userdata of the given size, push it
    /// onto the top of the stack, and set its destructor to the given function.
    /// It will return a pointer to the allocated userdata.
    pub fn lua_newuserdatadtor(
        L: *mut lua_State,
        size: usize,
        dtor: Option<extern "C-unwind" fn(*mut c_void)>,
    ) -> *mut c_void;

    /// Pushes a new buffer onto the top of the stack.
    ///
    /// This function will allocate a new buffer of the given size, push it onto
    /// the top of the stack, and return a pointer to the allocated buffer.
    pub fn lua_newbuffer(L: *mut lua_State, size: usize) -> *mut c_void;

    /// Indexes a table at the given index with the key at the top of the stack
    /// and pushes the value onto the stack.
    ///
    /// This function is equivalent to the Luau code `t[k]` where `t` is at the
    /// given index, and `k` is at the top of the stack. The key will be popped
    /// from the stack, and the resulting value will be pushed onto the stack.
    ///
    /// The type of the resulting value is returned.
    pub fn lua_gettable(L: *mut lua_State, idx: c_int) -> lua_Type;

    /// Indexes a table at the given index with the given string key and pushes
    /// the value onto the stack.
    ///
    /// This function is equivalent to the Luau code `t[k]` where `t` is at the
    /// given index, and `k` is the given string key. The resulting value will
    /// be pushed onto the stack.
    ///
    /// The type of the resulting value is returned.
    pub fn lua_getfield(L: *mut lua_State, idx: c_int, k: *const c_char) -> lua_Type;

    /// Raw-indexes a table at the given index with the given string key and
    /// pushes the value onto the stack.
    ///
    /// This function is equivalent to the Luau code `rawget(t, k)` where `t` is
    /// at the given index, and `k` is the given string key. The resulting
    /// value will be pushed onto the stack.
    ///
    /// The type of the resulting value is returned.
    pub fn lua_rawgetfield(L: *mut lua_State, idx: c_int, k: *const c_char) -> lua_Type;

    /// Raw-indexes a table at the given index with the key at the top of the
    /// stack and pushes the value onto the stack.
    ///
    /// This function is equivalent to the Luau code `rawget(t, k)` where `t` is
    /// at the given index, and `k` is at the top of the stack. The key will be
    /// popped from the stack, and the resulting value will be pushed onto the
    /// stack.
    ///
    /// The type of the resulting value is returned.
    pub fn lua_rawget(L: *mut lua_State, idx: c_int) -> lua_Type;

    /// Raw-indexes a table at the given index with the given integer key and
    /// pushes the value onto the stack.
    ///
    /// This function is equivalent to the Luau code `rawget(t, k)` where `t` is
    /// at the given index, and `k` is the given integer key. The resulting
    /// value will be pushed onto the stack.
    ///
    /// The type of the resulting value is returned.
    pub fn lua_rawgeti(L: *mut lua_State, idx: c_int, k: c_int) -> lua_Type;

    /// Creates a new table of the given size and pushes it onto the stack.
    pub fn lua_createtable(L: *mut lua_State, narr: c_int, nrec: c_int);

    /// Sets the readonly status of the table at the given index.
    pub fn lua_setreadonly(L: *mut lua_State, idx: c_int, enabled: c_int);

    /// Gets the readonly status of the table at the given index.
    pub fn lua_getreadonly(L: *mut lua_State, idx: c_int) -> c_int;

    /// Sets the safe environment status of the table at the given index.
    pub fn lua_setsafeenv(L: *mut lua_State, idx: c_int, enabled: c_int);

    /// Gets and pushes the metatable of the value at the given index onto the
    /// top of the stack.
    ///
    /// If the value at the given index does not have a metatable, this function
    /// will not push anything onto the stack and will return `0`. If the value
    /// does have a metatable, it will be pushed onto the stack and `1` will be
    /// returned.
    pub fn lua_getmetatable(L: *mut lua_State, idx: c_int) -> c_int;

    /// Gets the environment of the value at the given index and pushes it onto
    /// the top of the stack.
    ///
    /// If the value at the given index does not have an environment, this
    /// function will push `nil` onto the stack. If the value does have an
    /// environment, it will be pushed onto the stack.
    ///
    /// Only functions and threads have environments.
    pub fn lua_getfenv(L: *mut lua_State, idx: c_int);

    /// Sets a value in a table at the given index with the key at the top of
    /// stack.
    ///
    /// This function is equivalent to the Luau code `t[k] = v` where `t` is at
    /// the given index, `v` is at the top of the stack, and `k` is right below
    /// `v`.
    ///
    /// Both the key and value will be popped from the stack.
    pub fn lua_settable(L: *mut lua_State, idx: c_int);

    /// Sets a value in a table at the given index with the given string key.
    ///
    /// This function is equivalent to the Luau code `t[k] = v` where `t` is at
    /// the given index, `v` is at the top of the stack, and `k` is the given
    /// string key.
    ///
    /// The value will be popped from the stack.
    pub fn lua_setfield(L: *mut lua_State, idx: c_int, k: *const c_char);

    /// Raw-sets a value in a table at the given index with the given string
    /// key.
    ///
    /// This function is equivalent to the Luau code `rawset(t, k, v)` where `t`
    /// is at the given index, `v` is at the top of the stack, and `k` is the
    /// given string key.
    ///
    /// The value will be popped from the stack.
    pub fn lua_rawsetfield(L: *mut lua_State, idx: c_int, k: *const c_char);

    /// Raw-sets a value in a table at the given index with the key at the top
    /// of the stack.
    ///
    /// This function is equivalent to the Luau code `rawset(t, k, v)` where `t`
    /// is at the given index, `v` is at the top of the stack, and `k` is right
    /// below `v`.
    ///
    /// Both the key and value will be popped from the stack.
    pub fn lua_rawset(L: *mut lua_State, idx: c_int);

    /// Raw-sets a value in a table at the given index with the given integer
    /// key.
    ///
    /// This function is equivalent to the Luau code `rawset(t, k, v)` where `t`
    /// is at the given index, `v` is at the top of the stack, and `k` is the
    /// given integer key.
    ///
    /// The value will be popped from the stack.
    pub fn lua_rawseti(L: *mut lua_State, idx: c_int, k: c_int) -> c_int;

    /// Sets the metatable of the value at the given index to the value at the
    /// top of the stack.
    ///
    /// The metatable of tables and userdata can be set using this, but also the
    /// metatable of any other primitive type can be set.
    ///
    /// This function will set the metatable of the value at the given index
    /// to the value at the top of the stack, and pop the value from the stack.
    ///
    /// This function will always return `1`.
    pub fn lua_setmetatable(L: *mut lua_State, idx: c_int) -> c_int;

    /// Sets the environment of the value at the given index to the value at the
    /// top of the stack.
    ///
    /// This function will set the environment of the value at the given index
    /// to the value at the top of the stack, and pop the value from the stack.
    ///
    /// Only functions and threads have environments, so this function will only
    /// work for those types.
    ///
    /// This function will return `1` if the environment was set successfully,
    /// and `0` if the value at the given index does not have an environment.
    pub fn lua_setfenv(L: *mut lua_State, idx: c_int) -> c_int;

    /// Pushes a function constructed from bytecode onto the top of the stack.
    ///
    /// This function will push a function onto the top of the stack, which is
    /// constructed from the given bytecode. The bytecode is expected to be in
    /// the format produced by the Luau compiler.
    ///
    /// The `env` argument is the index of the environment table to use for the
    /// function. If `env` is `0`, the function will use the current environment
    /// of the state.
    pub fn luau_load(
        L: *mut lua_State,
        chunkname: *const c_char,
        data: *const c_char,
        size: usize,
        env: c_int,
    ) -> c_int;

    /// Calls a function on the stack, with the given number of arguments, and
    /// the given number of results.
    ///
    /// The function should be pushed onto the stack, and then the arguments
    /// should be pushed onto the stack above the function. The arguments and
    /// the function will be popped from the stack, and the results will be
    /// pushed onto the stack.
    ///
    /// The `nargs` argument is the number of arguments to pass to the function,
    /// and the `nresults` argument is the number of results to expect from the
    /// function. If there are more results than `nresults`, the additional
    /// results will be discarded. If there are fewer results than `nresults`,
    /// the missing results will be filled with `nil` values.
    ///
    /// If `nresults` is [`LUA_MULTRET`], all results will be pushed onto the
    /// stack.
    ///
    /// Any errors that occur during the call will be propagated and this
    /// function will not return. If the function yields, this function will
    /// return.
    pub fn lua_call(L: *mut lua_State, nargs: c_int, nresults: c_int);

    /// Calls a protected function on the stack, with the given number of
    /// arguments, and the given number of results.
    ///
    /// This function is similar to [`lua_call`], but in the case of an error,
    /// the error will be caught, the error value will be pushed onto the stack,
    /// and an error code will be returned.
    ///
    /// If `errfunc` is not `0`, it will be used as the index of the error
    /// handler function, which will be called with the error value on the stack
    /// and should return an error value. This function is usually used to add
    /// stack traces to errors, as the error handler function is called before
    /// the stack is unwound.
    ///
    /// When the call is successful, the results will be pushed onto the stack,
    /// and the function will return [`LUA_OK`].
    ///
    /// When the call errors, the error handler, if it exists, will be called,
    /// the error value will be pushed to the stack, and the function will
    /// return [`LUA_ERRRUN`], or if the error handler errored, [`LUA_ERRERR`],
    /// or if there was an allocation error [`LUA_ERRMEM`] will be returned.
    ///
    /// When the call yields, the function will return [`LUA_OK`]
    pub fn lua_pcall(
        L: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
    ) -> lua_Status;

    /// Yields the thread with the given number of results.
    ///
    /// This function will yield the thread, yielding the given number of
    /// results to the resumer. This function should only be called from return
    /// positions of C functions.
    pub fn lua_yield(L: *mut lua_State, nresults: c_int) -> c_int;

    /// TODO: Document this function.
    pub fn lua_break(L: *mut lua_State) -> c_int;

    /// Resumes a thread with the given number of arguments.
    ///
    /// This function will resume the thread with the given number of arguments.
    /// The arguments should be pushed onto the stack before calling resume.
    ///
    /// The `from` argument is used to preserve the C call depth and limit. It
    /// may be null, but this is bad practice.
    ///
    /// This function returns the status of the resumed thread after it stops
    /// executing.
    pub fn lua_resume(L: *mut lua_State, from: *mut lua_State, nargs: c_int) -> lua_Status;

    /// Resumes a thread with an error.
    ///
    /// This function is similar to [`lua_resume`], but it will resume the
    /// thread with an error. The error value is whatever is at the top of the
    /// stack.
    ///
    /// The `from` argument is used to preserve the C call depth and limit. It
    /// may be null, but this is bad practice.
    ///
    /// This function returns the status of the resumed thread after it stops
    /// executing.
    pub fn lua_resumeerror(L: *mut lua_State, from: *mut lua_State) -> lua_Status;

    /// Returns the status of the thread.
    pub fn lua_status(L: *mut lua_State) -> lua_Status;

    /// Returns if the thread is yieldable or not.
    ///
    /// Threads may not yield when doing so would yield across a C call without
    /// a continuation, such as during a metamethod call.
    pub fn lua_isyieldable(L: *mut lua_State) -> c_int;

    /// Returns the thread-data pointer of the given thread.
    ///
    /// This function returns the thread-data pointer of the given thread, which
    /// is null if the thread-data is unset, or the pointer previously set by
    /// [`lua_setthreaddata`].
    pub fn lua_getthreaddata(L: *mut lua_State) -> *mut c_void;

    /// Sets the thread-data pointer of the given thread.
    ///
    /// This function sets the thread-data pointer of the given thread to the
    /// given pointer. The thread-data pointer is a pointer that can be used to
    /// store arbitrary data associated with the thread. It is not used by Luau
    /// itself, and is intended for use by C extensions.
    pub fn lua_setthreaddata(L: *mut lua_State, data: *mut c_void);

    /// Returns the status of the given coroutine.
    ///
    /// This is equivalent to the Luau function `coroutine.status`.
    pub fn lua_costatus(L: *mut lua_State) -> lua_CoStatus;
}

/// Garbage collection operations that can be performed.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum lua_GCOp {
    /// Stops the incremental garbage collector.
    LUA_GCSTOP,

    /// Restarts the incremental garbage collector after a stop.
    LUA_GCRESTART,

    /// Runs a full garbage collection cycle.
    LUA_GCCOLLECT,

    /// Counts the heap size in kilobytes.
    LUA_GCCOUNT,

    /// Counts the remainder of the heap size in bytes, after the kilobytes.
    LUA_GCCOUNTB,

    /// Returns if the GC is running. This may still return true even if the GC
    /// is not actively collecting.
    LUA_GCISRUNNING,

    /// Performs an explicit GC step with the given size in kilobytes.
    LUA_GCSTEP,

    /// Sets the goal of the incremental garbage collector. The goal is given
    /// as a percentage of live data to all data, so 200% (the default) means
    /// that the heap is allowed to grow to twice the size of the live data.
    LUA_GCSETGOAL,

    /// Sets the step multiplier of the incremental garbage collector. The GC
    /// will try to collect at least this percentage of the heap size each step.
    LUA_GCSETSTEPMUL,

    /// Sets the step size of the incremental garbage collector. The GC will
    /// run a step after this many bytes are allocated.
    LUA_GCSETSTEPSIZE,
}

unsafe extern "C-unwind" {
    /// Performs a garbage collection operation.
    ///
    /// These operations are defined and documented in [`lua_GCOp`].
    pub fn lua_gc(L: *mut lua_State, what: lua_GCOp, data: c_int) -> c_int;

    /// Sets the memory category of the thread.
    ///
    /// All memory allocated by the thread will be counted towards this
    /// category, which can be used to track memory usage of different parts of
    /// the program.
    pub fn lua_setmemcat(L: *mut lua_State, category: c_int);

    /// Reads the number of bytes allocated to a given memory category.
    pub fn lua_totalbytes(L: *mut lua_State, category: c_int) -> usize;

    /// Throws an error.
    ///
    /// This function will throw an error with the top value on the stack being
    /// the error value. This function will not return.
    pub fn lua_error(L: *mut lua_State) -> !;

    /// Produces the next key and value pair from a table given the previous
    /// key.
    ///
    /// This function is equivalent to the Luau code `next(t, k)` where `t` is
    /// the table at the given index, and `k` is the value at the top of the
    /// stack. The key will be popped from the stack, and the resulting key and
    /// value will be pushed onto the stack.
    ///
    /// If the table has no more key-value pairs, this function will return `0`
    /// and will not push a key and value onto the stack, but the previous key
    /// will still be popped from the stack.
    ///
    /// A typical traversal of a table using this function would look like this:
    ///
    /// ```c
    /// // table is at index t
    /// lua_pushnil(L); // push nil as the first key
    /// while (lua_next(L, t) != 0) {
    ///     // key is at index -2, value is at index -1
    ///     lua_pop(L, 1); // remove value, keep key for next iteration
    /// }
    /// ```
    ///
    /// This function, while still usable, is not recommended for use. Instead,
    /// use [`lua_rawiter`].
    pub fn lua_next(L: *mut lua_State, idx: c_int) -> c_int;

    /// Produces the next key and value pair from a table given the previous
    /// iteration index.
    ///
    /// This function will iterate over the table at the given index, starting
    /// from the array section, then iterating over the hashmap section. After
    /// each call, the function returns an iteration index that should be passed
    /// to the next call to continue iterating, and it will return `-1` when
    /// iteration is complete.
    ///
    /// The function will push the key and value onto the stack.
    ///
    /// A typical traversal of a table using this function would look like this:
    ///
    /// ```c
    /// // table is at index t
    /// int iteration_index = 0; // start iteration with 0
    /// while ((iteration_index = lua_rawiter(L, t, iteration_index)) != -1) {
    ///     // key is at index -2, value is at index -1
    ///     lua_pop(L, 2); // remove key and value
    /// }
    /// ```
    pub fn lua_rawiter(L: *mut lua_State, idx: c_int, iteration_index: c_int) -> c_int;

    /// Concatenates `n` values from the stack into a string.
    ///
    /// This function pops `n` values from the top of the stack, concatenates
    /// them into a string, and pushes the resulting string onto the top of the
    /// stack. When `n` is `0` an empty string is pushed onto the stack.
    pub fn lua_concat(L: *mut lua_State, n: c_int);

    /// Encodes a pointer such that it remains unique but no longer points to
    /// the original location.
    ///
    /// This is useful for sandboxing pointers exposed to Luau.
    pub fn lua_encodepointer(L: *mut lua_State, p: usize) -> usize;

    /// Returns a high-precision timestamp in seconds.
    ///
    /// The returned timestamp doesn't have a defined epoch, but can be used to
    /// measure duration with sub-microsecond precision.
    pub fn lua_clock() -> c_double;

    /// Sets the tag of a userdata at the given index.
    pub fn lua_setuserdatatag(L: *mut lua_State, idx: c_int, tag: c_int) -> c_int;
}

/// The type of destructor functions for userdata.
pub type lua_Destructor = extern "C-unwind" fn(L: *mut lua_State, userdata: *mut c_void);

unsafe extern "C-unwind" {
    /// Sets the destructor of a userdata tag.
    ///
    /// For all userdata with the same tag, the destructor will be called when
    /// the userdata is collected by the garbage collector.
    pub fn lua_setuserdatadtor(L: *mut lua_State, tag: c_int, dtor: Option<lua_Destructor>);

    /// Gets the destructor of a userdata tag.
    ///
    /// For all userdata with the same tag, the destructor will be called when
    /// the userdata is collected by the garbage collector.
    pub fn lua_getuserdatadtor(L: *mut lua_State, tag: c_int) -> Option<lua_Destructor>;

    /// Sets the metatable that userdata with the given tag will have when
    /// created with [`lua_newuserdatataggedwithmetatable`].
    ///
    /// The metatable will be popped from the top of the stack.
    pub fn lua_setuserdatametatable(L: *mut lua_State, tag: c_int);

    /// Gets the metatable that userdata with the given tag will have when
    /// created with [`lua_newuserdatataggedwithmetatable`].
    ///
    /// The metatable will be pushed onto the top of the stack.
    pub fn lua_getuserdatametatable(L: *mut lua_State, tag: c_int) -> c_int;

    /// Sets the name of a light userdata tag.
    pub fn lua_setlightuserdataname(L: *mut lua_State, tag: c_int, name: *const c_char);

    /// Gets the name of a light userdata tag.
    pub fn lua_getlightuserdataname(L: *mut lua_State, tag: c_int) -> *const c_char;

    /// Clones the function at the given index and pushes it onto the stack.
    ///
    /// The function must be a Luau function, it cannot be a C function. All
    /// of the function's upvalues will be copied (they reference the same
    /// upvalues). The function will have the environment of the current
    /// thread.
    pub fn lua_clonefunction(L: *mut lua_State, idx: c_int);

    /// Clears the table at the given index.
    ///
    /// The table will be cleared of all values, but it will retain its
    /// allocated size and metatable.
    pub fn lua_cleartable(L: *mut lua_State, idx: c_int);

    /// Clones the table at the given index and pushes it onto the stack.
    ///
    /// All the table's keys and values will be copied, the cloned table will
    /// have the same metatable and will not be readonly. The copy is shallow.
    pub fn lua_clonetable(L: *mut lua_State, idx: c_int);

    /// Returns the memory allocation function of the state.
    pub fn lua_getallocf(L: *mut lua_State) -> Option<lua_Alloc>;
}

/// A placeholder value that is unique from any created reference.
pub const LUA_NOREF: c_int = -1;

/// A reference that is used to refer to a nil value.
pub const LUA_REFNIL: c_int = 0;

unsafe extern "C-unwind" {
    /// Creates a reference to the value at the given index.
    ///
    /// This function returns an integer reference to the value at the given
    /// index that can be used to push the value onto the stack later. While
    /// the value is referenced, it will not be collected by the garbage
    /// collector.
    ///
    /// References can be pushed onto a stack using [`lua_getref`] and must be
    /// released using [`lua_unref`].
    pub fn lua_ref(L: *mut lua_State, idx: c_int) -> c_int;

    /// Releases a reference to a value.
    ///
    /// This function will release the given reference created by [`lua_ref`].
    /// If the reference is [`LUA_NOREF`] or [`LUA_REFNIL`] this function will
    /// do nothing.
    pub fn lua_unref(L: *mut lua_State, ref_: c_int);
}

/// Pushes the value of the given reference onto the stack.
///
/// This function will push the value of the given reference onto the stack and
/// return the type of the value. If the reference is [`LUA_NOREF`] or
/// [`LUA_REFNIL`], this function will push `nil` onto the stack and return
/// [`LUA_TNIL`].
pub unsafe fn lua_getref(L: *mut lua_State, r#ref: c_int) -> lua_Type {
    unsafe { lua_rawgeti(L, LUA_REGISTRYINDEX, r#ref) }
}

/// Attempts to convert the value at the given index to a number, and returns
/// it.
///
/// If the value at the given index cannot be converted to a number, this
/// function will return `0.0`.
///
/// This function will leave numbers as is, and will transform strings which
/// can be parsed as numbers into numbers. This will modify the stack.
pub unsafe fn lua_tonumber(L: *mut lua_State, idx: c_int) -> lua_Number {
    unsafe { lua_tonumberx(L, idx, null_mut()) }
}

/// Attempts to convert the value at the given index to an integer, and returns
/// it.
///
/// This function operates in a similar way as [`lua_tonumber`], but it has the
/// final result floored to an integer. While it will convert a string into a
/// number on the stack, it will not modify the stack value into an integer.
pub unsafe fn lua_tointeger(L: *mut lua_State, idx: c_int) -> lua_Integer {
    unsafe { lua_tointegerx(L, idx, null_mut()) }
}

/// Attempts to convert the value at the given index to an unsigned integer, and
/// returns it.
///
/// This function operates in the same way as [`lua_tointeger`], but it will
/// return an unsigned integer instead. If the input is negative, the output
/// is undefined.
pub unsafe fn lua_tounsigned(L: *mut lua_State, idx: c_int) -> lua_Unsigned {
    unsafe { lua_tounsignedx(L, idx, null_mut()) }
}

/// Pops `n` number of values from the stack.
pub unsafe fn lua_pop(L: *mut lua_State, n: c_int) {
    unsafe { lua_settop(L, -(n) - 1) }
}

/// Creates a new table and pushes it onto the stack.
pub unsafe fn lua_newtable(L: *mut lua_State) {
    unsafe { lua_createtable(L, 0, 0) }
}

/// Creates a new userdata of the given size and pushes it onto the stack.
pub unsafe fn lua_newuserdata(L: *mut lua_State, size: usize) -> *mut c_void {
    unsafe { lua_newuserdatatagged(L, size, 0) }
}

/// Returns the length of the string at the given index in bytes.
pub unsafe fn lua_strlen(L: *mut lua_State, idx: c_int) -> c_int {
    unsafe { lua_objlen(L, idx) }
}

/// Returns if the value at the given index is a function.
pub unsafe fn lua_isfunction(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TFUNCTION }
}

/// Returns if the value at the given index is a table.
pub unsafe fn lua_istable(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TTABLE }
}

/// Returns if the value at the given index is a light userdata.
pub unsafe fn lua_islightuserdata(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TLIGHTUSERDATA }
}

/// Returns if the value at the given index is nil.
pub unsafe fn lua_isnil(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TNIL }
}

/// Returns if the value at the given index is a boolean.
pub unsafe fn lua_isboolean(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TBOOLEAN }
}

/// Returns if the value at the given index is a vector.
pub unsafe fn lua_isvector(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TVECTOR }
}

/// Returns if the value at the given index is a thread.
pub unsafe fn lua_isthread(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TTHREAD }
}

/// Returns if the value at the given index is a buffer.
pub unsafe fn lua_isbuffer(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TBUFFER }
}

/// Returns if the value at the given index is none.
pub unsafe fn lua_isnone(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TNONE }
}

/// Returns if the value at the given index is none or nil.
pub unsafe fn lua_isnoneornil(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) as c_int <= LUA_TNIL as c_int }
}

/// Pushes a literal string onto the stack.
pub unsafe fn lua_pushliteral(L: *mut lua_State, s: &'static [u8]) {
    unsafe { lua_pushlstring(L, s.as_ptr() as _, s.len()) }
}

/// Pushes a C function onto the stack.
pub unsafe fn lua_pushcfunction(L: *mut lua_State, r#fn: lua_CFunction, debugname: *const c_char) {
    unsafe { lua_pushcclosurek(L, r#fn, debugname, 0, None) }
}

/// Pushes a C closure onto the stack.
pub unsafe fn lua_pushcclosure(
    L: *mut lua_State,
    r#fn: lua_CFunction,
    debugname: *const c_char,
    nup: c_int,
) {
    unsafe { lua_pushcclosurek(L, r#fn, debugname, nup, None) }
}

/// Pushes a light userdata onto the stack.
pub unsafe fn lua_pushlightuserdata(L: *mut lua_State, p: *mut c_void) {
    unsafe { lua_pushlightuserdatatagged(L, p, 0) }
}

/// Sets a global variable with the given name.
pub unsafe fn lua_setglobal(L: *mut lua_State, name: *const c_char) {
    unsafe { lua_setfield(L, LUA_GLOBALSINDEX, name) }
}

/// Gets a global variable with the given name and pushes it onto the stack.
pub unsafe fn lua_getglobal(L: *mut lua_State, name: *const c_char) -> lua_Type {
    unsafe { lua_getfield(L, LUA_GLOBALSINDEX, name) }
}

/// Attempts to convert the value at the given index to a string, and return it.
///
/// This function attempts to convert the value at the given index to a
/// string, returning a pointer to the string if successful, or null if the
/// conversion failed.
///
/// If the value is a string, it will be returned as is. If the value is a
/// number, it will be converted into a string and the stack will be
/// modified.
pub unsafe fn lua_tostring(L: *mut lua_State, idx: c_int) -> *const c_char {
    unsafe { lua_tolstring(L, idx, null_mut()) }
}

/// TODO: Document this struct.
#[repr(C)]
pub struct lua_Debug {
    pub name: *const c_char,
    pub what: *const c_char,
    pub source: *const c_char,
    pub short_src: *const c_char,
    pub linedefined: c_int,
    pub currentline: c_int,
    pub nupvals: c_uchar,
    pub nparams: c_uchar,
    pub isvararg: c_char,
    pub userdata: *mut c_void,

    pub ssbuf: [c_char; LUA_IDSIZE as usize],
}

/// TODO: Document this type.
pub type lua_Hook = extern "C-unwind" fn(L: *mut lua_State, ar: *mut lua_Debug);

unsafe extern "C-unwind" {
    /// Returns the stack depth, or the number of calls on the stack.
    pub fn lua_stackdepth(L: *mut lua_State) -> c_int;

    /// TODO: Document this function.
    pub fn lua_getinfo(
        L: *mut lua_State,
        level: c_int,
        what: *const c_char,
        ar: *mut lua_Debug,
    ) -> c_int;

    /// TODO: Document this function.
    pub fn lua_getargument(L: *mut lua_State, level: c_int, n: c_int) -> c_int;

    /// TODO: Document this function.
    pub fn lua_getlocal(L: *mut lua_State, level: c_int, n: c_int) -> *const c_char;

    /// TODO: Document this function.
    pub fn lua_setlocal(L: *mut lua_State, level: c_int, n: c_int) -> *const c_char;

    /// TODO: Document this function.
    pub fn lua_getupvalue(L: *mut lua_State, funcindex: c_int, n: c_int) -> *const c_char;

    /// TODO: Document this function.
    pub fn lua_setupvalue(L: *mut lua_State, funcindex: c_int, n: c_int) -> *const c_char;

    /// TODO: Document this function.
    pub fn lua_singlestep(L: *mut lua_State, enabled: c_int);

    /// TODO: Document this function.
    pub fn lua_breakpoint(
        L: *mut lua_State,
        funcindex: c_int,
        line: c_int,
        enabled: c_int,
    ) -> c_int;
}

/// TODO: Document this type.
pub type lua_Coverage = extern "C-unwind" fn(
    context: *mut c_void,
    function: *const c_char,
    linedefined: c_int,
    depth: c_int,
    hits: *const c_int,
    size: usize,
);

unsafe extern "C-unwind" {
    /// TODO: Document this function.
    pub fn lua_getcoverage(
        L: *mut lua_State,
        funcindex: c_int,
        context: *mut c_void,
        callback: lua_Coverage,
    );

    /// TODO: Document this function.
    pub fn lua_debugtrace(L: *mut lua_State) -> *const c_char;
}

/// TODO: Document this struct.
#[repr(C)]
pub struct lua_Callbacks {
    pub userdata: *mut c_void,

    pub interrupt: Option<extern "C-unwind" fn(L: *mut lua_State, gc: c_int)>,

    pub panic: Option<extern "C-unwind" fn(L: *mut lua_State, errcode: c_int)>,

    pub userthread: Option<extern "C-unwind" fn(LP: *mut lua_State, L: *mut lua_State)>,

    pub useratom: Option<extern "C-unwind" fn(s: *const c_char, l: usize) -> i16>,

    pub debugbreak: Option<lua_Hook>,

    pub debugstep: Option<lua_Hook>,

    pub debuginterrupt: Option<lua_Hook>,

    pub debugprotectederror: Option<lua_Hook>,

    pub onallocate: Option<extern "C-unwind" fn(L: *mut lua_State, osize: usize, nsize: usize)>,
}

unsafe extern "C-unwind" {
    /// TODO: Document this function.
    pub fn lua_callbacks(L: *mut lua_State) -> *mut lua_Callbacks;
}
