#![no_std]

pub use core::ffi::{c_int, c_short, c_void};

#[link(name = "coconut")]
unsafe extern "C" {
    pub safe fn exit(v: u32) -> !;
    pub unsafe fn write(i: i32, p: *const u8, len: usize) -> isize;

    pub fn malloc(n: usize) -> *mut c_void;
    pub fn free(_: *mut c_void);
    pub fn realloc(_: *mut c_void, _: usize) -> *mut c_void;
    pub fn calloc(_: usize, _: usize) -> *mut c_void;
    pub fn posix_memalign(_: *mut *mut c_void, align: usize, len: usize) -> c_int;
}
