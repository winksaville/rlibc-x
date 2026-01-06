#![no_std]

use core::arch::asm;

pub mod stubs;

#[allow(unused)]
use core::panic::PanicInfo;

// Syscall numbers for Linux x86_64
const SYS_READ: u64 = 0;
const SYS_WRITE: u64 = 1;
const SYS_BRK: u64 = 12;
const SYS_EXIT: u64 = 60;

// Simple heap state for malloc/realloc/free
static mut HEAP_START: *mut u8 = core::ptr::null_mut();
static mut HEAP_END: *mut u8 = core::ptr::null_mut();
static mut HEAP_CURRENT: *mut u8 = core::ptr::null_mut();

#[inline(always)]
pub unsafe fn syscall0(n: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") n,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

#[inline(always)]
pub unsafe fn syscall1(n: u64, arg1: u64) -> u64 {
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

#[inline(always)]
pub unsafe fn syscall2(n: u64, arg1: u64, arg2: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg1,
            in("rsi") arg2,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

#[inline(always)]
pub unsafe fn syscall3(n: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

#[inline(always)]
pub unsafe fn syscall4(n: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

#[inline(always)]
pub unsafe fn syscall5(n: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            in("r8") arg5,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

#[inline(always)]
pub unsafe fn syscall6(
    n: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            in("r8") arg5,
            in("r9") arg6,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

// --- Public API ---

pub fn exit(code: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code as u64);
    }
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn write(fd: i32, buf: *const u8, count: usize) -> isize {
    unsafe { syscall3(SYS_WRITE, fd as u64, buf as u64, count as u64) as isize }
}

#[unsafe(no_mangle)]
pub extern "C" fn read(fd: i32, buf: *mut u8, count: usize) -> isize {
    unsafe { syscall3(SYS_READ, fd as u64, buf as u64, count as u64) as isize }
}

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
    // Simple bump allocator doesn't free individual allocations
}

// --- Runtime (only for no_std apps using the "runtime" feature) ---

#[cfg(all(feature = "runtime", not(test)))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(101)
}

#[cfg(feature = "runtime")]
#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

#[cfg(feature = "runtime")]
unsafe extern "Rust" {
    safe fn main();
}

#[cfg(feature = "runtime")]
extern "C" fn _start_rust() -> ! {
    init_heap();
    main();
    exit(0)
}

#[cfg(feature = "runtime")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    // Kernel enters with RSP aligned to 16 bytes, pointing to argc
    // call pushes 8-byte return address, so callee sees RSP % 16 == 8
    core::arch::naked_asm!(
        "mov rbx, rsp",               // save original RSP (points to argc)
        "mov rdi, [rbx]",             // rdi = argc
        "lea rsi, [rbx + 8]",         // rsi = argv
        "lea rdx, [rsi + rdi*8 + 8]", // rdx = envp
        "call {start_rust}",
        "ud2",
        start_rust = sym _start_rust,
    );
}
