use std::ffi::CStr;

use crate::{Function, Stack, ThreadData};

pub struct Library<MD, TD: ThreadData<MD>>(pub Vec<(&'static str, LibraryItem<MD, TD>)>);

impl<MD, TD: ThreadData<MD>> Library<MD, TD> {
    pub fn push(&self, stack: &Stack<MD, TD>) {
        stack.reserve(2);
        stack.push_table();

        for (name, item) in &self.0 {
            stack.push_string(name);
            item.push(stack);
            stack.table_set_raw(-3);
        }
    }
}

pub enum LibraryItem<MD, TD: ThreadData<MD>> {
    Library(Library<MD, TD>),
    Constant(LibraryConstant),
    Function(Function<MD, TD>),
}

impl<MD, TD: ThreadData<MD>> LibraryItem<MD, TD> {
    pub fn push(&self, stack: &Stack<MD, TD>) {
        match self {
            LibraryItem::Library(lib) => lib.push(stack),
            LibraryItem::Constant(constant) => constant.push(stack),
            LibraryItem::Function(func) => {
                stack.reserve(1);
                stack.push_function(func)
            }
        }
    }
}

pub enum LibraryConstant {
    Bool(bool),
    Number(f64),
    String(&'static str),
    Vector(f32, f32, f32),
}

impl LibraryConstant {
    pub fn push<MD, TD: ThreadData<MD>>(&self, stack: &Stack<MD, TD>) {
        stack.reserve(1);

        match self {
            LibraryConstant::Bool(value) => stack.push_boolean(*value),
            LibraryConstant::Number(value) => stack.push_number(*value),
            LibraryConstant::String(value) => stack.push_string(value),
            LibraryConstant::Vector(x, y, z) => stack.push_vector((*x, *y, *z)),
        }
    }
}

impl<MD, TD: ThreadData<MD>> From<Library<MD, TD>> for LibraryItem<MD, TD> {
    fn from(lib: Library<MD, TD>) -> Self {
        LibraryItem::Library(lib)
    }
}

impl<MD, TD: ThreadData<MD>> From<LibraryConstant> for LibraryItem<MD, TD> {
    fn from(constant: LibraryConstant) -> Self {
        LibraryItem::Constant(constant)
    }
}

impl<MD, TD: ThreadData<MD>> From<Function<MD, TD>> for LibraryItem<MD, TD> {
    fn from(func: Function<MD, TD>) -> Self {
        LibraryItem::Function(func)
    }
}
