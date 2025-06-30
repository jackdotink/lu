pub use derive::Userdata;
pub use sys;

mod alloc;
mod compiler;
mod context;
mod extra;
mod library;
mod stack;
mod state;
mod thread;
mod userdata;

pub use alloc::{DefaultAllocator, LuauAllocator};
pub use compiler::{Bytecode, CompileResult, Compiler};
pub use context::{Context, FnReturn};
pub use extra::{Function, Ref, Status, Type};
pub use library::{Library, LibraryConstant, LibraryItem};
pub use stack::Stack;
pub use state::State;
pub use thread::{Thread, ThreadMain, ThreadRef};
pub use userdata::{Userdata, unique_tag};

#[allow(unused)]
pub trait Config: Sized {
    type Allocator: LuauAllocator;

    type MainData;
    type ThreadData: ThreadData<Self>;
}

pub trait ThreadData<C: Config>: Sized {
    fn new(parent: &Thread<C>, thread: &Thread<C>) -> Self;
}

impl<C: Config> ThreadData<C> for () {
    fn new(_parent: &Thread<C>, _thread: &Thread<C>) -> Self {}
}
