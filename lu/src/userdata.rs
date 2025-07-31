use std::sync::atomic::{AtomicU32, Ordering};

use crate::{Config, Context, FnReturn, Function, Status};

pub fn unique_tag() -> u32 {
    static COUNT: AtomicU32 = AtomicU32::new(1);

    let tag = COUNT.fetch_add(1, Ordering::Relaxed);
    assert!(
        tag < sys::LUA_UTAG_LIMIT as u32,
        "userdata tag limit exceeded",
    );

    tag
}

pub trait Userdata {
    fn tag() -> u32;
    fn name() -> &'static str;
}

#[derive(Default)]
pub struct Methods<C: Config> {
    pub(crate) methods: Vec<(&'static str, Function<C>)>,
}

impl<C: Config> Methods<C> {
    pub fn with_method(mut self, name: &'static str, func: Function<C>) -> Self {
        self.methods.push((name, func));
        self
    }

    pub fn with_method_norm(
        self,
        name: &'static str,
        func: extern "C-unwind" fn(ctx: Context<C>) -> FnReturn,
    ) -> Self {
        self.with_method(name, Function::norm(name, func))
    }

    pub fn with_method_cont(
        self,
        name: &'static str,
        func: extern "C-unwind" fn(ctx: Context<C>) -> FnReturn,
        cont: extern "C-unwind" fn(ctx: Context<C>, status: Status) -> FnReturn,
    ) -> Self {
        self.with_method(name, Function::cont(name, func, cont))
    }
}
