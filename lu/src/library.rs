use crate::{Config, Context, FnReturn, Function, Stack};

#[derive(Default)]
pub struct Library<C: Config>(Vec<(&'static str, LibraryItem<C>)>);

impl<C: Config> Library<C> {
    pub fn with(mut self, name: &'static str, item: impl Into<LibraryItem<C>>) -> Self {
        self.0.push((name, item.into()));
        self
    }

    pub fn with_constant(self, name: &'static str, constant: impl Into<LibraryConstant>) -> Self {
        self.with(name, LibraryItem::Constant(constant.into()))
    }

    pub fn with_function(self, name: &'static str, func: Function<C>) -> Self {
        self.with(name, LibraryItem::Function(func))
    }

    pub fn with_function_norm(
        self,
        name: &'static str,
        func: extern "C-unwind" fn(ctx: Context<C>) -> FnReturn,
    ) -> Self {
        self.with(name, Function::norm(name, func))
    }

    pub fn with_function_cont(
        self,
        name: &'static str,
        func: extern "C-unwind" fn(ctx: Context<C>) -> FnReturn,
        cont: extern "C-unwind" fn(ctx: Context<C>, status: crate::Status) -> FnReturn,
    ) -> Self {
        self.with(name, Function::cont(name, func, cont))
    }

    pub fn push(&self, stack: &Stack<C>) {
        stack.reserve(2);
        stack.push_table();

        for (name, item) in &self.0 {
            stack.push_string(name);
            item.push(stack);
            stack.table_set_raw(-3);
        }
    }
}

pub enum LibraryItem<C: Config> {
    Library(Library<C>),
    Constant(LibraryConstant),
    Function(Function<C>),
}

impl<C: Config> From<Library<C>> for LibraryItem<C> {
    fn from(lib: Library<C>) -> Self {
        LibraryItem::Library(lib)
    }
}

impl<C: Config> From<LibraryConstant> for LibraryItem<C> {
    fn from(constant: LibraryConstant) -> Self {
        LibraryItem::Constant(constant)
    }
}

impl<C: Config> From<Function<C>> for LibraryItem<C> {
    fn from(func: Function<C>) -> Self {
        LibraryItem::Function(func)
    }
}

impl<C: Config> LibraryItem<C> {
    pub fn push(&self, stack: &Stack<C>) {
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
    pub fn push<C: Config>(&self, stack: &Stack<C>) {
        stack.reserve(1);

        match self {
            LibraryConstant::Bool(value) => stack.push_boolean(*value),
            LibraryConstant::Number(value) => stack.push_number(*value),
            LibraryConstant::String(value) => stack.push_string(value),
            LibraryConstant::Vector(x, y, z) => stack.push_vector((*x, *y, *z)),
        }
    }
}

impl From<bool> for LibraryConstant {
    fn from(value: bool) -> Self {
        LibraryConstant::Bool(value)
    }
}

impl From<f64> for LibraryConstant {
    fn from(value: f64) -> Self {
        LibraryConstant::Number(value)
    }
}

impl From<&'static str> for LibraryConstant {
    fn from(value: &'static str) -> Self {
        LibraryConstant::String(value)
    }
}

impl From<(f32, f32, f32)> for LibraryConstant {
    fn from(value: (f32, f32, f32)) -> Self {
        LibraryConstant::Vector(value.0, value.1, value.2)
    }
}
