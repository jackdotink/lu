use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
};

use crate::{Stack, ThreadData, ThreadRef, Type, Userdata};

#[repr(transparent)]
pub struct FnReturn(i32);

#[repr(transparent)]
pub struct Context<MD, TD: ThreadData<MD>>(NonNull<sys::lua_State>, PhantomData<(MD, TD)>);

impl<MD, TD: ThreadData<MD>> Deref for Context<MD, TD> {
    type Target = Stack<MD, TD>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl<MD, TD: ThreadData<MD>> Context<MD, TD> {
    pub fn arg_error(&self, narg: u32, reason: &CStr) -> ! {
        unsafe { sys::luaL_argerror(self.as_ptr(), narg as _, reason.as_ptr()) }
    }

    pub fn arg_type_error(&self, narg: u32, type_name: &CStr) -> ! {
        unsafe { sys::luaL_typeerror(self.as_ptr(), narg as _, type_name.as_ptr()) }
    }

    pub fn arg_boolean(&self, narg: u32) -> bool {
        self.to_boolean(narg as _)
            .unwrap_or_else(|| self.arg_type_error(narg, c"boolean"))
    }

    pub fn arg_boolean_opt(&self, narg: u32) -> Option<bool> {
        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::Boolean => Some(self.to_boolean_unchecked(narg as _)),
            _ => self.arg_type_error(narg, c"boolean or nil"),
        }
    }

    pub fn arg_number(&self, narg: u32) -> f64 {
        self.to_number(narg as _)
            .unwrap_or_else(|| self.arg_type_error(narg, c"number"))
    }

    pub fn arg_number_opt(&self, narg: u32) -> Option<f64> {
        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::Number => Some(self.to_number_unchecked(narg as _)),
            _ => self.arg_type_error(narg, c"number or nil"),
        }
    }

    pub fn arg_vector(&self, narg: u32) -> (f32, f32, f32) {
        self.to_vector(narg as _)
            .unwrap_or_else(|| self.arg_type_error(narg, c"vector"))
    }

    pub fn arg_vector_opt(&self, narg: u32) -> Option<(f32, f32, f32)> {
        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::Vector => Some(unsafe { self.to_vector_unchecked(narg as _) }),
            _ => self.arg_type_error(narg, c"vector or nil"),
        }
    }

    pub fn arg_string_slice(&self, narg: u32) -> &[u8] {
        self.to_string_slice(narg as _)
            .unwrap_or_else(|| self.arg_type_error(narg, c"string"))
    }

    pub fn arg_string_slice_opt(&self, narg: u32) -> Option<&[u8]> {
        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::String => Some(unsafe { self.to_string_slice_unchecked(narg as _) }),
            _ => self.arg_type_error(narg, c"string or nil"),
        }
    }

    pub fn arg_string_str(&self, narg: u32) -> &str {
        self.to_string_str(narg as _)
            .unwrap_or_else(|| self.arg_type_error(narg, c"utf-8 string"))
    }

    pub fn arg_string_str_opt(&self, narg: u32) -> Option<&str> {
        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::String => {
                if let Some(s) = unsafe { self.to_string_str_unchecked(narg as _) } {
                    Some(s)
                } else {
                    self.arg_type_error(narg, c"utf-8 string")
                }
            }
            _ => self.arg_type_error(narg, c"utf-8 string or nil"),
        }
    }

    pub fn arg_table(&self, narg: u32) {
        if !self.is_table(narg as _) {
            self.arg_type_error(narg, c"table");
        }
    }

    pub fn arg_table_opt(&self, narg: u32) -> Option<()> {
        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::Table => Some(()),
            _ => self.arg_type_error(narg, c"table or nil"),
        }
    }

    pub fn arg_function(&self, narg: u32) {
        if !self.is_function(narg as _) {
            self.arg_type_error(narg, c"function");
        }
    }

    pub fn arg_function_opt(&self, narg: u32) -> Option<()> {
        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::Function => Some(()),
            _ => self.arg_type_error(narg, c"function or nil"),
        }
    }

    pub fn arg_userdata<T: Userdata>(&self, narg: u32) -> &RefCell<T> {
        self.to_userdata(narg as _).unwrap_or_else(|| {
            let type_name = CString::new(T::name()).expect("userdata name contains null byte");

            self.arg_type_error(narg, type_name.as_c_str())
        })
    }

    pub fn arg_userdata_opt<T: Userdata>(&self, narg: u32) -> Option<&RefCell<T>> {
        fn error<MD, TD: ThreadData<MD>, T: Userdata>(ctx: &Context<MD, TD>, narg: u32) -> ! {
            let type_name = CString::new(format!("{} or nil", T::name()))
                .expect("userdata name contains null byte");

            ctx.arg_type_error(narg, type_name.as_c_str())
        }

        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::Userdata => Some(
                self.to_userdata(narg as _)
                    .unwrap_or_else(|| error::<_, _, T>(self, narg)),
            ),
            _ => error::<_, _, T>(self, narg),
        }
    }

    pub fn arg_thread(&self, narg: u32) -> ThreadRef<MD, TD> {
        self.to_thread(narg as _)
            .unwrap_or_else(|| self.arg_type_error(narg, c"thread"))
    }

    pub fn arg_thread_opt(&self, narg: u32) -> Option<ThreadRef<MD, TD>> {
        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::Thread => Some(unsafe { self.to_thread_unchecked(narg as _) }),
            _ => self.arg_type_error(narg, c"thread or nil"),
        }
    }

    pub fn arg_buffer(&self, narg: u32) -> (*mut u8, usize) {
        self.to_buffer(narg as _)
            .unwrap_or_else(|| self.arg_type_error(narg, c"buffer"))
    }

    pub fn arg_buffer_opt(&self, narg: u32) -> Option<(*mut u8, usize)> {
        match self.type_of(narg as _) {
            Type::Nil | Type::None => None,
            Type::Buffer => Some(unsafe { self.to_buffer_unchecked(narg as _) }),
            _ => self.arg_type_error(narg, c"buffer or nil"),
        }
    }

    pub fn push_upvalue(&self, nup: u32) {
        self.push_copy(sys::lua_upvalueindex(nup as _));
    }

    pub fn set_upvalue(&self, nup: u32) {
        self.replace(sys::lua_upvalueindex(nup as _));
    }

    pub fn ret(self) -> FnReturn {
        FnReturn(0)
    }

    pub fn ret_with(self, n: u32) -> FnReturn {
        FnReturn(n as _)
    }

    pub fn yld(self) -> FnReturn {
        unsafe { FnReturn(sys::lua_yield(self.as_ptr(), 0)) }
    }

    pub fn yld_with(self, n: u32) -> FnReturn {
        unsafe { FnReturn(sys::lua_yield(self.as_ptr(), n as _)) }
    }
}
