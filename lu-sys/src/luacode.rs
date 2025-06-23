#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_double, c_float, c_int, c_void};

/// A constant that can be configured during compilation to enable constant
/// folding and other optimizations.
pub type lua_CompileConstant = *mut c_void;

/// A callback to retrieve the type of a member in a library, which is used to
/// provide type information during native codegen.
///
/// Waiting on https://github.com/luau-lang/luau/discussions/1885 for more
/// details on how this works.
pub type lua_LibraryMemberTypeCallback =
    extern "C" fn(library: *const c_char, member: *const c_char) -> c_int;

/// Allows the configuration of a constant in a library at compile time. This
/// enables the compiler to optimize code with constant folding and other
/// optimizations.
pub type lua_LibraryMemberConstantCallback = extern "C" fn(
    library: *const c_char,
    member: *const c_char,
    constant: *mut lua_CompileConstant,
);

/// Options to configure the Luau compiler with.
///
/// All memory pointed to by this struct must be valid for the duration of the
/// [`luau_compile`] call that this struct is passed to.
#[repr(C)]
pub struct lua_CompileOptions {
    /// What kind of optimizations the compiler should perform.
    ///
    /// 0: No optimizations.
    /// 1: Optimizations that do not impact debugging or debugging information.
    /// 2: Optimizations that may impact debugging or debugging information.
    pub optimizationLevel: c_int,

    /// The amount of debug information to include in the compiled bytecode.
    ///
    /// 0: No debug information.
    /// 1: Line info and function names, sufficient for backtraces.
    /// 2: Full debug information including variable names.
    pub debugLevel: c_int,

    /// The type information to include in the compiled bytecode.
    ///
    /// 0: Emit type information only for native codegen.
    /// 1: Emit type information for all code.
    pub typeInfoLevel: c_int,

    /// The amount of coverage information to include in the compiled bytecode.
    ///
    /// 0: No coverage information.
    /// 1: Coverage information for statements.
    /// 2: Coverage information for statements and expressions.
    pub coverageLevel: c_int,

    /// An alternative global used to construct vectors in addition to
    /// `vector.create`.
    ///
    /// This field is the library name. The constructor name is in
    /// [`lua_CompileOptions::vectorCtor`].
    ///
    /// The full configured vector constructor is
    /// `<vectorLib>.<vectorCtor>(x, y, z)`.
    pub vectorLib: *const c_char,

    /// The name of the alternative vector constructor function.
    ///
    /// This field is the constructor name. The library name is in
    /// [`lua_CompileOptions::vectorLib`].
    ///
    /// The full configured vector constructor is
    /// `<vectorLib>.<vectorCtor>(x, y, z)`.
    pub vectorCtor: *const c_char,

    /// An alternative name for the vector type in addition to `vector`.
    pub vectorType: *const c_char,

    /// An array of global names that are mutable globals.
    ///
    /// The import optimization will be disabled for these globals. This array
    /// is null-terminated.
    pub mutableGlobals: *const *const c_char,

    /// Waiting to document alongside [`lua_LibraryMemberTypeCallback`]
    pub userdataTypes: *const *const c_char,

    /// An array of global names that are libraries with known members.
    ///
    /// This array is null-terminated.
    pub librariesWithKnownMembers: *const *const c_char,

    /// A callback to retrieve the type of a member in a library.
    pub libraryMemberTypeCb: Option<lua_LibraryMemberTypeCallback>,

    /// A callback to retrieve the constant value of a member in a library.
    pub libraryMemberConstantCb: Option<lua_LibraryMemberConstantCallback>,

    /// An array of global or library function names that are normally built-in
    /// to the language, but are disabled for this compilation.
    ///
    /// This array contains members like "print" or "table.insert". This array
    /// is null-terminated.
    pub disabledBuiltins: *const *const c_char,
}

unsafe extern "C" {
    /// Compiles the given source code into Luau bytecode.
    ///
    /// The `source` parameter is a pointer to the source code as a string of
    /// length `size`. The `options` parameter is a pointer to a
    /// [`lua_CompileOptions`] struct that configures the compilation process.
    /// The `outsize` parameter is a pointer to a variable that will receive the
    /// size of the compiled bytecode.
    ///
    /// If compilation is successful, this function returns a pointer to the
    /// compiled bytecode. If compilation fails, the error is encoded in the
    /// returned bytecode. This can be detected by checking if the first byte
    /// of the returned bytecode is `0`, which indicates an error. The remaining
    /// bytes will contain the error message.
    ///
    /// All produced bytecode, both successful and non-successful, can be loaded
    /// into a Luau VM using [`luau_load`](crate::luau_load).To drop, the returned pointer should
    /// be passed into [`libc::free`].
    pub fn luau_compile(
        source: *const c_char,
        size: usize,
        options: *mut lua_CompileOptions,
        outsize: *mut usize,
    ) -> *mut c_char;

    /// Sets the given compile constant to `nil`.
    pub fn luau_set_compile_constant_nil(constant: *mut lua_CompileConstant);

    /// Sets the given compile constant to a boolean value.
    pub fn luau_set_compile_constant_boolean(constant: *mut lua_CompileConstant, b: c_int);

    /// Sets the given compile constant to a number value.
    pub fn luau_set_compile_constant_number(constant: *mut lua_CompileConstant, n: c_double);

    /// Sets the given compile constant to a vector value. This function always
    /// takes four components, but when loaded into a Luau VM the last component
    /// will be ignored if the VM is in 3-wide mode.
    pub fn luau_set_compile_constant_vector(
        constant: *mut lua_CompileConstant,
        x: c_float,
        y: c_float,
        z: c_float,
        w: c_float,
    );

    /// Sets the given compile constant to a string value.
    ///
    /// The string is a pointer to a string of length `l`.
    pub fn luau_set_compile_constant_string(
        constant: *mut lua_CompileConstant,
        s: *const c_char,
        l: usize,
    );
}
