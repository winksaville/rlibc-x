#![no_std]

use core::arch::asm;

#[allow(unused)] // Probably needed because panic hander is not used directly
use core::panic::PanicInfo;

// Syscall numbers for Linux x86_64
const SYS_EXIT: u64 = 60;
const SYS_BRK: u64 = 12;

// Simple heap state for malloc/realloc/free
static mut HEAP_START: *mut u8 = core::ptr::null_mut();
static mut HEAP_END: *mut u8 = core::ptr::null_mut();
static mut HEAP_CURRENT: *mut u8 = core::ptr::null_mut();

#[inline(always)]
unsafe fn syscall1(n: u64, arg1: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg1,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

pub fn exit(code: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code as u64);
    }
    loop {}
}

fn brk(addr: *mut u8) -> *mut u8 {
    unsafe { syscall1(SYS_BRK, addr as u64) as *mut u8 }
}

fn init_heap() {
    unsafe {
        if HEAP_START.is_null() {
            HEAP_START = brk(core::ptr::null_mut());
            HEAP_END = HEAP_START;
            HEAP_CURRENT = HEAP_START;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    unsafe {
        init_heap();

        // Align to 16 bytes
        let aligned_size = (size + 15) & !15;
        let new_current = HEAP_CURRENT.add(aligned_size);

        // Grow heap if needed
        if new_current > HEAP_END {
            let grow_size = (aligned_size + 4095) & !4095; // Page align
            let new_end = brk(HEAP_END.add(grow_size));
            if new_end <= HEAP_END {
                return core::ptr::null_mut(); // brk failed
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
    // Simple implementation: just allocate new and copy
    // (we don't track allocation sizes, so we can't do better without more infrastructure)
    let new_ptr = malloc(size);
    if !new_ptr.is_null() {
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, new_ptr, size);
        }
    }
    new_ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn free(_ptr: *mut u8) {
    // Simple bump allocator doesn't free individual allocations
    // Memory is only returned when the process exits
}

#[cfg(not(test))] // Need as zed thinks this could be a test :(
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(101)
}

#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

unsafe extern "Rust" {
    safe fn main();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    main();
    exit(0)
}
