use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    Bytecode, Context, FnReturn, Function, Ref, Status, Thread, ThreadData, ThreadMain, ThreadRef,
    Type, Userdata,
};

#[repr(transparent)]
pub struct Stack<MD, TD: ThreadData<MD>>(
    pub(crate) NonNull<sys::lua_State>,
    pub(crate) PhantomData<(MD, TD)>,
);

impl<MD, TD: ThreadData<MD>> Stack<MD, TD> {
    pub fn as_ptr(&self) -> *mut sys::lua_State {
        self.0.as_ptr()
    }

    pub fn main(&self) -> ThreadMain<MD, TD> {
        let ptr = unsafe { NonNull::new_unchecked(sys::lua_mainthread(self.as_ptr())) };

        ThreadMain {
            thread: Thread(ptr, PhantomData),
        }
    }

    pub fn thread(&self) -> &Thread<MD, TD> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn error(&self) -> ! {
        unsafe { sys::lua_error(self.as_ptr()) }
    }

    pub fn error_msg(&self, msg: impl AsRef<[u8]>) -> ! {
        self.push_string(msg);
        self.error();
    }

    pub fn abs_idx(&self, idx: i32) -> i32 {
        unsafe { sys::lua_absindex(self.as_ptr(), idx as _) }
    }

    pub fn get_top(&self) -> i32 {
        unsafe { sys::lua_gettop(self.as_ptr()) as _ }
    }

    pub fn set_top(&self, idx: i32) {
        unsafe { sys::lua_settop(self.as_ptr(), idx as _) }
    }

    pub fn pop(&self, n: i32) {
        unsafe { sys::lua_pop(self.as_ptr(), n as _) }
    }

    pub fn remove(&self, idx: i32) {
        unsafe { sys::lua_remove(self.as_ptr(), idx as _) }
    }

    pub fn insert(&self, idx: i32) {
        unsafe { sys::lua_insert(self.as_ptr(), idx as _) }
    }

    pub fn replace(&self, idx: i32) {
        unsafe { sys::lua_replace(self.as_ptr(), idx as _) }
    }

    pub fn reserve(&self, n: i32) {
        unsafe { sys::lua_rawcheckstack(self.as_ptr(), n as _) }
    }

    pub fn xmove(&self, to: &Thread<MD, TD>, n: u32) {
        unsafe { sys::lua_xmove(self.as_ptr(), to.as_ptr(), n as _) }
    }

    pub fn xpush(&self, to: &Thread<MD, TD>, idx: i32) {
        unsafe { sys::lua_xpush(self.as_ptr(), to.as_ptr(), idx as _) }
    }

    pub fn push_copy(&self, idx: i32) {
        unsafe { sys::lua_pushvalue(self.as_ptr(), idx as _) }
    }

    pub fn push_nil(&self) {
        unsafe { sys::lua_pushnil(self.as_ptr()) }
    }

    pub fn push_boolean(&self, value: bool) {
        unsafe { sys::lua_pushboolean(self.as_ptr(), value as _) }
    }

    pub fn push_light_userdata<T>(&self, value: *mut T) {
        unsafe { sys::lua_pushlightuserdata(self.as_ptr(), value.cast()) }
    }

    pub fn push_number(&self, value: f64) {
        unsafe { sys::lua_pushnumber(self.as_ptr(), value) }
    }

    pub fn push_vector(&self, v: (f32, f32, f32)) {
        unsafe { sys::lua_pushvector(self.as_ptr(), v.0, v.1, v.2) }
    }

    pub fn push_string(&self, s: impl AsRef<[u8]>) {
        let s = s.as_ref();
        unsafe { sys::lua_pushlstring(self.as_ptr(), s.as_ptr().cast(), s.len()) }
    }

    pub fn push_table(&self) {
        unsafe { sys::lua_createtable(self.as_ptr(), 0, 0) };
    }

    pub fn push_table_with(&self, narr: u32, nrec: u32) {
        unsafe { sys::lua_createtable(self.as_ptr(), narr as _, nrec as _) };
    }

    pub fn push_extern_closure_cont(
        &self,
        name: &CStr,
        nups: u32,
        func: extern "C-unwind" fn(ctx: Context<MD, TD>) -> FnReturn,
        cont: extern "C-unwind" fn(ctx: Context<MD, TD>, status: Status) -> FnReturn,
    ) {
        unsafe {
            let func = std::mem::transmute::<
                extern "C-unwind" fn(Context<MD, TD>) -> FnReturn,
                extern "C-unwind" fn(*mut sys::lua_State) -> i32,
            >(func);

            let cont = std::mem::transmute::<
                extern "C-unwind" fn(Context<MD, TD>, Status) -> FnReturn,
                extern "C-unwind" fn(*mut sys::lua_State, sys::lua_Status) -> i32,
            >(cont);

            sys::lua_pushcclosurek(self.as_ptr(), func, name.as_ptr(), nups as _, Some(cont));
        }
    }

    pub fn push_extern_closure(
        &self,
        name: &CStr,
        nups: u32,
        func: extern "C-unwind" fn(ctx: Context<MD, TD>) -> FnReturn,
    ) {
        unsafe {
            let func = std::mem::transmute::<
                extern "C-unwind" fn(Context<MD, TD>) -> FnReturn,
                extern "C-unwind" fn(*mut sys::lua_State) -> i32,
            >(func);

            sys::lua_pushcclosure(self.as_ptr(), func, name.as_ptr(), nups as _);
        }
    }

    pub fn push_extern_function_cont(
        &self,
        name: &CStr,
        func: extern "C-unwind" fn(ctx: Context<MD, TD>) -> FnReturn,
        cont: extern "C-unwind" fn(ctx: Context<MD, TD>, status: Status) -> FnReturn,
    ) {
        self.push_extern_closure_cont(name, 0, func, cont);
    }

    pub fn push_extern_function(
        &self,
        name: &CStr,
        func: extern "C-unwind" fn(ctx: Context<MD, TD>) -> FnReturn,
    ) {
        self.push_extern_closure(name, 0, func);
    }

    pub fn push_function(&self, func: &Function<MD, TD>) {
        let name = CString::new(func.name()).expect("function name contains null byte");

        match func {
            Function::Normal { func, .. } => self.push_extern_function(name.as_c_str(), *func),
            Function::Continuation { func, cont, .. } => {
                self.push_extern_function_cont(name.as_c_str(), *func, *cont)
            }
        }
    }

    pub fn push_bytecode(&self, name: &CStr, bytecode: Bytecode) {
        unsafe {
            sys::luau_load(
                self.as_ptr(),
                name.as_ptr(),
                bytecode.ptr(),
                bytecode.len(),
                0,
            )
        };
    }

    pub fn push_userdata<T: Userdata>(&self, value: T) {
        let tag = T::tag();
        let size = size_of::<RefCell<T>>();

        unsafe {
            #[cfg(debug_assertions)]
            if sys::lua_getuserdatadtor(self.as_ptr(), tag as _).is_none() {
                panic!("attempt to push unregistered userdata type: {}", T::name());
            }

            let ptr = sys::lua_newuserdatataggedwithmetatable(self.as_ptr(), size, tag as _);
            let ptr = ptr.cast::<RefCell<T>>();

            ptr.write(RefCell::new(value));
        }
    }

    pub fn push_thread(&self, thread: &Thread<MD, TD>) {
        if thread.as_ptr() == self.as_ptr() {
            unsafe { sys::lua_pushthread(self.as_ptr()) };
        } else {
            let stack = thread.stack();
            stack.reserve(1);
            unsafe { sys::lua_pushthread(stack.as_ptr()) };
            stack.xmove(self.thread(), 1);
        }
    }

    pub fn push_thread_new(&self) -> ThreadRef<MD, TD> {
        let thread = unsafe { sys::lua_newthread(self.as_ptr()) };

        let thread = Thread(NonNull::new(thread).unwrap(), PhantomData);
        let th_ref = self.to_ref(-1);

        ThreadRef {
            thread,
            _thref: th_ref,
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn push_buffer(&self, size: usize) -> (*mut u8, usize) {
        let ptr = unsafe { sys::lua_newbuffer(self.as_ptr(), size) };
        (ptr.cast(), size)
    }

    pub fn push_ref(&self, r: &Ref<MD, TD>) -> Type {
        unsafe { Type::from(sys::lua_getref(self.as_ptr(), r.1 as _)) }
    }

    pub fn type_of(&self, idx: i32) -> Type {
        unsafe { Type::from(sys::lua_type(self.as_ptr(), idx as _)) }
    }

    pub fn is_none(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::None
    }

    pub fn is_nil(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Nil
    }

    pub fn is_boolean(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Boolean
    }

    pub fn is_light_userdata(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::LightUserdata
    }

    pub fn is_number(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Number
    }

    pub fn is_vector(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Vector
    }

    pub fn is_string(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::String
    }

    pub fn is_table(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Table
    }

    pub fn is_function(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Function
    }

    pub fn is_userdata<T: Userdata>(&self, idx: i32) -> bool {
        unsafe { sys::lua_userdatatag(self.as_ptr(), idx) == (T::tag() as _) }
    }

    pub fn is_thread(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Thread
    }

    pub fn is_buffer(&self, idx: i32) -> bool {
        self.type_of(idx) == Type::Buffer
    }

    pub fn to_boolean_unchecked(&self, idx: i32) -> bool {
        unsafe { sys::lua_toboolean(self.as_ptr(), idx as _) != 0 }
    }

    pub fn to_boolean(&self, idx: i32) -> Option<bool> {
        if self.is_boolean(idx) {
            Some(self.to_boolean_unchecked(idx))
        } else {
            None
        }
    }

    pub fn to_light_userdata_unchecked<T>(&self, idx: i32) -> *mut T {
        unsafe { sys::lua_touserdata(self.as_ptr(), idx as _).cast() }
    }

    pub fn to_light_userdata<T>(&self, idx: i32) -> Option<*mut T> {
        if self.is_light_userdata(idx) {
            Some(self.to_light_userdata_unchecked(idx))
        } else {
            None
        }
    }

    pub fn to_number_unchecked(&self, idx: i32) -> f64 {
        unsafe { sys::lua_tonumber(self.as_ptr(), idx as _) }
    }

    pub fn to_number(&self, idx: i32) -> Option<f64> {
        if self.is_number(idx) {
            Some(self.to_number_unchecked(idx))
        } else {
            None
        }
    }

    pub unsafe fn to_vector_unchecked(&self, idx: i32) -> (f32, f32, f32) {
        let ptr = unsafe { sys::lua_tovector(self.as_ptr(), idx) };
        unsafe { (ptr.read(), ptr.add(1).read(), ptr.add(2).read()) }
    }

    pub fn to_vector(&self, idx: i32) -> Option<(f32, f32, f32)> {
        let ptr = unsafe { sys::lua_tovector(self.as_ptr(), idx) };

        if !ptr.is_null() {
            Some(unsafe { (ptr.read(), ptr.add(1).read(), ptr.add(2).read()) })
        } else {
            None
        }
    }

    pub unsafe fn to_string_slice_unchecked(&self, idx: i32) -> &[u8] {
        let mut len = 0;
        let ptr = unsafe { sys::lua_tolstring(self.as_ptr(), idx as _, &mut len) };

        unsafe { std::slice::from_raw_parts(ptr.cast(), len) }
    }

    pub fn to_string_slice(&self, idx: i32) -> Option<&[u8]> {
        if self.is_string(idx) {
            Some(unsafe { self.to_string_slice_unchecked(idx) })
        } else {
            None
        }
    }

    pub unsafe fn to_string_str_unchecked(&self, idx: i32) -> Option<&str> {
        std::str::from_utf8(unsafe { self.to_string_slice_unchecked(idx) }).ok()
    }

    pub fn to_string_str(&self, idx: i32) -> Option<&str> {
        std::str::from_utf8(self.to_string_slice(idx)?).ok()
    }

    pub fn to_userdata<T: Userdata>(&self, idx: i32) -> Option<&RefCell<T>> {
        unsafe {
            sys::lua_touserdatatagged(self.as_ptr(), idx, T::tag() as _)
                .cast::<RefCell<T>>()
                .as_ref()
        }
    }

    pub unsafe fn to_thread_unchecked(&self, idx: i32) -> ThreadRef<MD, TD> {
        let ptr = unsafe { sys::lua_tothread(self.as_ptr(), idx as _) };

        let thread = unsafe { Thread(NonNull::new_unchecked(ptr), PhantomData) };
        let th_ref = self.to_ref(idx);

        ThreadRef {
            thread,
            _thref: th_ref,
        }
    }

    pub fn to_thread(&self, idx: i32) -> Option<ThreadRef<MD, TD>> {
        let ptr = unsafe { sys::lua_tothread(self.as_ptr(), idx as _) };

        if !ptr.is_null() {
            let thread = Thread(NonNull::new(ptr).unwrap(), PhantomData);
            let th_ref = self.to_ref(idx);

            Some(ThreadRef {
                thread,
                _thref: th_ref,
            })
        } else {
            None
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn to_buffer_unchecked(&self, idx: i32) -> (*mut u8, usize) {
        let mut len = 0;
        let ptr = unsafe { sys::lua_tobuffer(self.as_ptr(), idx as _, &mut len) };

        (ptr.cast(), len)
    }

    pub fn to_buffer(&self, idx: i32) -> Option<(*mut u8, usize)> {
        let mut len = 0;
        let ptr = unsafe { sys::lua_tobuffer(self.as_ptr(), idx as _, &mut len) };

        if !ptr.is_null() {
            Some((ptr.cast(), len))
        } else {
            None
        }
    }

    pub fn to_ref(&self, idx: i32) -> Ref<MD, TD> {
        Ref(self.main(), unsafe {
            sys::lua_ref(self.as_ptr(), idx) as _
        })
    }

    pub fn table_get(&self, tblidx: i32) {
        unsafe { sys::lua_gettable(self.as_ptr(), tblidx) };
    }

    pub fn table_set(&self, tblidx: i32) {
        unsafe { sys::lua_settable(self.as_ptr(), tblidx) };
    }

    pub fn table_get_field(&self, tblidx: i32, field: &CStr) {
        unsafe { sys::lua_getfield(self.as_ptr(), tblidx, field.as_ptr()) };
    }

    pub fn table_set_field(&self, tblidx: i32, field: &CStr) {
        unsafe { sys::lua_setfield(self.as_ptr(), tblidx, field.as_ptr()) };
    }

    pub fn table_get_raw(&self, tblidx: i32) {
        unsafe { sys::lua_rawget(self.as_ptr(), tblidx) };
    }

    pub fn table_set_raw(&self, tblidx: i32) {
        unsafe { sys::lua_rawset(self.as_ptr(), tblidx) };
    }

    pub fn table_get_raw_i(&self, tblidx: i32, i: u32) {
        unsafe { sys::lua_rawgeti(self.as_ptr(), tblidx, i as _) };
    }

    pub fn table_set_raw_i(&self, tblidx: i32, i: u32) {
        unsafe { sys::lua_rawseti(self.as_ptr(), tblidx, i as _) };
    }

    pub fn table_get_raw_field(&self, tblidx: i32, field: &CStr) {
        unsafe { sys::lua_rawgetfield(self.as_ptr(), tblidx, field.as_ptr()) };
    }

    pub fn table_set_raw_field(&self, tblidx: i32, field: &CStr) {
        unsafe { sys::lua_rawsetfield(self.as_ptr(), tblidx, field.as_ptr()) };
    }

    pub fn table_set_readonly(&self, tblidx: i32, readonly: bool) {
        unsafe { sys::lua_setreadonly(self.as_ptr(), tblidx, readonly as _) };
    }

    pub fn table_get_readonly(&self, tblidx: i32) -> bool {
        unsafe { sys::lua_getreadonly(self.as_ptr(), tblidx) != 0 }
    }

    pub fn len(&self, idx: i32) -> u32 {
        unsafe { sys::lua_objlen(self.as_ptr(), idx as _) as u32 }
    }

    pub fn iter(&self, tblidx: i32, mut func: impl FnMut()) {
        let mut iterindex = 0;
        loop {
            let nextindex = unsafe { sys::lua_rawiter(self.as_ptr(), tblidx, iterindex) };

            if nextindex == -1 {
                break;
            } else {
                iterindex = nextindex;
                func();
                self.pop(2);
            }
        }
    }
}
