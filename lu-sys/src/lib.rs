//! Rust bindings for the Luau C API.
//!
//! # WIP
//!
//! This crate is a work in progress and not yet complete. Bindings have not
//! been written for all functions and documentation has not been written for
//! all functions that have bindings.
//!
//! * `luacodegen.h` (native code generation) - Not bound, not documented.
//! * `luacode.h` (bytecode compilation) - Fully bound, partially documented.
//! * `Require.h` (require functionality) - Not bound, not documented.
//! * `luaconf.h` (VM configuration) - Fully bound, not documented.
//! * `lualib.h` (aux VM functionality) - Fully bound, not documented.
//! * `lua.h` (core VM functionality) - Fully bound, documented.
//!
//! # Bindings
//!
//! All types and functions in the Luau C API are bound with the exception of
//! types and functions that utilize `...` or `va_list`. The Luau C API makes
//! extensive use of function-like macros, which have been closely reimplemented
//! in this crate as functions.
//!
//! # Stability and versioning
//!
//! Luau versions itself with an incrementing integer which increases with each
//! weekly release. As an example, at the time of writing, the latest Luau
//! version is `0.679`.
//!
//! This crate will track these versions, releasing a new version of the crate
//! weekly to match the latest Luau versions. When a breaking change occurs,
//! either in the Luau C API or in the bindings, a new major version will be
//! released.
//!
//! In summary, major versions will be released for breaking changes, minor
//! versions track the latest Luau version, and patch versions will be used only
//! for emergency bugfixes.
//!
//! # Portability
//!
//! Luau explicitly supports Windows, Linux, macOS, FreeBSD, iOS, and Android.
//! As such, this crate explicitly supports those platforms as well. As of time
//! of writing, the crate has only been tested on Linux. Please open an issue
//! if you encounter any problems on other platforms.

mod lua;
mod luacode;
mod luaconf;
mod lualib;

pub use lua::*;
pub use luacode::*;
pub use luaconf::*;
pub use lualib::*;
