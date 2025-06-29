use std::alloc::{Layout, alloc, dealloc, realloc};

pub trait LuauAllocator {
    fn alloc(&mut self, nsize: usize) -> *mut u8;
    fn realloc(&mut self, ptr: *mut u8, osize: usize, nsize: usize) -> *mut u8;
    fn dealloc(&mut self, ptr: *mut u8, osize: usize);
}

#[derive(Default)]
pub struct DefaultAllocator;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl LuauAllocator for DefaultAllocator {
    fn alloc(&mut self, nsize: usize) -> *mut u8 {
        let layout = Layout::from_size_align(nsize, 16).unwrap();
        unsafe { alloc(layout) }
    }

    fn realloc(&mut self, ptr: *mut u8, osize: usize, nsize: usize) -> *mut u8 {
        let layout = Layout::from_size_align(osize, 16).unwrap();
        unsafe { realloc(ptr, layout, nsize) }
    }

    fn dealloc(&mut self, ptr: *mut u8, osize: usize) {
        let layout = Layout::from_size_align(osize, 16).unwrap();
        unsafe { dealloc(ptr, layout) }
    }
}
