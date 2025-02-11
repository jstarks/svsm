#[no_mangle]
pub fn exit(v: u32) -> ! {
    syscall::exit(v)
}

#[no_mangle]
pub unsafe fn write(i: i32, p: *const u8, len: usize) -> isize {
    unsafe {
        match syscall::write(std::mem::transmute(&i), std::slice::from_raw_parts(p, len)) {
            Ok(v) => v as isize,
            Err(e) => e as isize,
        }
    }
}
