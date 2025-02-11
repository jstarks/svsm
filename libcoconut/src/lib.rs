use core::ffi::c_void;
use core::ptr::null_mut;

#[no_mangle]
pub extern "C" fn exit(v: i32) -> ! {
    syscall::exit(v as u32)
}

#[no_mangle]
pub unsafe extern "C" fn write(i: i32, p: *const u8, len: usize) -> isize {
    unsafe {
        match syscall::write(
            core::mem::transmute(&i),
            core::slice::from_raw_parts(p, len),
        ) {
            Ok(v) => v as isize,
            Err(e) => e as isize,
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    extern "C" {
        fn main(argc: i32, argv: *const *const ()) -> i32;
    }
    syscall::exit(unsafe { main(0, core::ptr::null()) } as u32);
}

const HEAP_LEN: usize = 0x100000;
#[repr(C, align(4096))]
struct Heap([core::sync::atomic::AtomicU8; HEAP_LEN]);
static HEAP: Heap = Heap([const { core::sync::atomic::AtomicU8::new(0) }; HEAP_LEN]);
static mut HEAP_OFFSET: usize = 0;

fn alloc_align(n: usize, align: usize) -> *mut c_void {
    let mut offset = unsafe {HEAP_OFFSET};
    let align = align.max(8);
    let align_mask = align - 1;
    offset += align_mask;
    offset &= !align_mask;
    if offset + n + align >= HEAP_LEN {
        return null_mut();
    }
    let header = unsafe { &mut *HEAP.0.as_ptr().byte_add(offset + align - 8).cast_mut().cast::<usize>() };
    *header = n;
    unsafe { HEAP_OFFSET = offset + align + n };
    unsafe { HEAP.0.as_ptr().byte_add(offset + align).cast_mut().cast()}
}

#[no_mangle]
pub extern "C" fn malloc(n: usize) -> *mut c_void {
    alloc_align(n, 16)
}

#[no_mangle]
pub extern "C" fn free(_: *mut c_void) {
}

fn alloc_size(p: *const c_void) -> usize {
    let header = unsafe { &*p.byte_sub(8).cast::<usize>() };
    *header
}

#[no_mangle]
pub extern "C" fn realloc(o: *mut c_void, len: usize) -> *mut c_void {
    let p = malloc(len);
    if !p.is_null() && !o.is_null() {
        unsafe { std::ptr::copy_nonoverlapping(o.cast_const().cast::<u8>(), p.cast::<u8>(), alloc_size(o)) };
    }
    p
}

#[no_mangle]
pub extern "C" fn calloc(a: usize, b: usize) -> *mut c_void {
    let p = malloc(a * b);
    if !p.is_null() {
        unsafe { p.write_bytes(0, a * b) };
    }
    p
}

#[no_mangle]
pub extern "C" fn posix_memalign(r: *mut *mut c_void, align: usize, len: usize) -> i32 {
    let p = alloc_align(len, align);
    if p.is_null() {
        return 3;
    }
    unsafe { r.write(p) };
    0
}
