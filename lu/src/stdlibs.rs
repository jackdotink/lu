use crate::{Library, Stack, ThreadMain};

pub struct Base;

impl<MainData, ThreadData> Library<MainData, ThreadData> for Base {
    fn name() -> &'static str {
        panic!("base library has no name")
    }

    fn push(_stack: &Stack<MainData, ThreadData>) {
        panic!("stdlib cannot be pushed, only opened")
    }

    fn open(thread: &ThreadMain<MainData, ThreadData>) {
        unsafe { sys::luaopen_base(thread.as_ptr()) };
    }
}

macro_rules! stdlib {
    ($name: ident, $strname: ident, $open: ident) => {
        pub struct $name;

        impl<MainData, ThreadData> Library<MainData, ThreadData> for $name {
            fn name() -> &'static str {
                const {
                    match (sys::$strname).to_str() {
                        Ok(name) => name,
                        Err(_) => unreachable!(),
                    }
                }
            }

            fn push(_stack: &Stack<MainData, ThreadData>) {
                panic!("stdlib cannot be pushed, only opened")
            }

            fn open(thread: &ThreadMain<MainData, ThreadData>) {
                unsafe { sys::$open(thread.as_ptr()) };
            }
        }
    };
}

stdlib!(Coroutine, LUA_COLIBNAME, luaopen_coroutine);
stdlib!(Table, LUA_TABLIBNAME, luaopen_table);
stdlib!(Os, LUA_OSLIBNAME, luaopen_os);
stdlib!(String, LUA_STRLIBNAME, luaopen_string);
stdlib!(Bit, LUA_BITLIBNAME, luaopen_bit32);
stdlib!(Buffer, LUA_BUFFERLIBNAME, luaopen_buffer);
stdlib!(Utf8, LUA_UTF8LIBNAME, luaopen_utf8);
stdlib!(Math, LUA_MATHLIBNAME, luaopen_math);
stdlib!(Debug, LUA_DBLIBNAME, luaopen_debug);
stdlib!(Vector, LUA_VECLIBNAME, luaopen_vector);

pub struct StdLibrary;

impl<MainData, ThreadData> Library<MainData, ThreadData> for StdLibrary {
    fn name() -> &'static str {
        panic!("stdlib has no name")
    }

    fn push(_stack: &Stack<MainData, ThreadData>) {
        panic!("stdlib cannot be pushed, only opened")
    }

    fn open(thread: &ThreadMain<MainData, ThreadData>) {
        Base::open(thread);
        Coroutine::open(thread);
        Table::open(thread);
        Os::open(thread);
        String::open(thread);
        Bit::open(thread);
        Buffer::open(thread);
        Utf8::open(thread);
        Math::open(thread);
        Vector::open(thread);
        Buffer::open(thread);
    }
}
