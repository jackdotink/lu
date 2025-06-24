use std::ffi::c_char;
use std::ptr::null;

#[derive(Default, Clone, Copy)]
pub enum OptimizationLevel {
    None,
    Some,
    #[default]
    Full,
}

impl From<OptimizationLevel> for i32 {
    fn from(level: OptimizationLevel) -> Self {
        match level {
            OptimizationLevel::None => 0,
            OptimizationLevel::Some => 1,
            OptimizationLevel::Full => 2,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum DebugInfoLevel {
    None,
    #[default]
    Some,
    Full,
}

impl From<DebugInfoLevel> for i32 {
    fn from(level: DebugInfoLevel) -> Self {
        match level {
            DebugInfoLevel::None => 0,
            DebugInfoLevel::Some => 1,
            DebugInfoLevel::Full => 2,
        }
    }
}

#[derive(Default)]
pub struct Compiler {
    optimization_level: OptimizationLevel,
    debug_level: DebugInfoLevel,
}

impl Compiler {
    pub fn with_optimization_level(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    pub fn with_debug_level(mut self, level: DebugInfoLevel) -> Self {
        self.debug_level = level;
        self
    }

    pub fn compile(&self, source: &[u8]) -> CompileResult {
        let mut options = sys::lua_CompileOptions {
            optimizationLevel: self.optimization_level.into(),
            debugLevel: self.debug_level.into(),
            typeInfoLevel: 0,
            coverageLevel: 0,
            vectorLib: null(),
            vectorCtor: null(),
            vectorType: null(),
            mutableGlobals: null(),
            userdataTypes: null(),
            librariesWithKnownMembers: null(),
            libraryMemberTypeCb: None,
            libraryMemberConstantCb: None,
            disabledBuiltins: null(),
        };

        let mut len = 0;
        let ptr = unsafe {
            sys::luau_compile(source.as_ptr().cast(), source.len(), &mut options, &mut len)
        };

        CompileResult { ptr, len }
    }
}

pub struct CompileResult {
    ptr: *mut c_char,
    len: usize,
}

impl Drop for CompileResult {
    fn drop(&mut self) {
        unsafe { libc::free(self.ptr.cast()) }
    }
}

impl CompileResult {
    pub fn bytecode(&self) -> Bytecode {
        Bytecode(unsafe { std::slice::from_raw_parts(self.ptr.cast(), self.len) })
    }
}

pub struct Bytecode<'a>(&'a [u8]);

impl Bytecode<'_> {
    pub fn ptr(&self) -> *const c_char {
        self.0.as_ptr().cast()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn result(&self) -> Result<(), &str> {
        if self.0.first().copied().unwrap() == 0 {
            Err(unsafe { std::str::from_utf8_unchecked(&self.0[1..]) })
        } else {
            Ok(())
        }
    }
}
