//! Memory allocation (malloc, free, etc.)

use crate::syscall::{syscall1, SYS_BRK};

// Heap state
static mut HEAP_START: *mut u8 = core::ptr::null_mut();
static mut HEAP_END: *mut u8 = core::ptr::null_mut();
static mut HEAP_CURRENT: *mut u8 = core::ptr::null_mut();

fn brk(addr: *mut u8) -> *mut u8 {
    unsafe { syscall1(SYS_BRK, addr as u64) as *mut u8 }
}

pub fn init_heap() {
    unsafe {
        HEAP_START = brk(core::ptr::null_mut());
        HEAP_END = HEAP_START;
        HEAP_CURRENT = HEAP_START;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    unsafe {
        let aligned_size = (size + 15) & !15;
        let new_current = HEAP_CURRENT.add(aligned_size);

        if new_current > HEAP_END {
            let grow_size = (aligned_size + 4095) & !4095;
            let new_end = brk(HEAP_END.add(grow_size));
            if new_end <= HEAP_END {
                return core::ptr::null_mut();
            }
            HEAP_END = new_end;
        }

        let ptr = HEAP_CURRENT;
        HEAP_CURRENT = new_current;
        ptr
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    if ptr.is_null() {
        return malloc(size);
    }
    let new_ptr = malloc(size);
    if !new_ptr.is_null() {
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, new_ptr, size);
        }
    }
    new_ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(nmemb: usize, size: usize) -> *mut u8 {
    let total = nmemb.saturating_mul(size);
    let ptr = malloc(total);
    if !ptr.is_null() {
        unsafe {
            core::ptr::write_bytes(ptr, 0, total);
        }
    }
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn free(_ptr: *mut u8) {
    // Bump allocator doesn't free individual allocations
}

#[unsafe(no_mangle)]
pub extern "C" fn posix_memalign(memptr: *mut *mut u8, alignment: usize, size: usize) -> i32 {
    // Simple implementation: over-allocate and align
    if !alignment.is_power_of_two() || alignment < core::mem::size_of::<*mut u8>() {
        return 22; // EINVAL
    }
    let ptr = malloc(size + alignment);
    if ptr.is_null() {
        return 12; // ENOMEM
    }
    let aligned = ((ptr as usize + alignment - 1) & !(alignment - 1)) as *mut u8;
    unsafe {
        *memptr = aligned;
    }
    0
}

// Memory operations needed by std
#[unsafe(no_mangle)]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        core::ptr::copy_nonoverlapping(src, dest, n);
    }
    dest
}

#[unsafe(no_mangle)]
pub extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        core::ptr::copy(src, dest, n);
    }
    dest
}

#[unsafe(no_mangle)]
pub extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    unsafe {
        core::ptr::write_bytes(dest, c as u8, n);
    }
    dest
}

#[unsafe(no_mangle)]
pub extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = unsafe { *s1.add(i) };
        let b = unsafe { *s2.add(i) };
        if a != b {
            return a as i32 - b as i32;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    memcmp(s1, s2, n)
}
