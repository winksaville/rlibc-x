//! Stub implementations for libc symbols required by std.
//!
//! These stubs print their name to stderr and exit with code 99.
//! Implement real versions as needed and remove from here.

use crate::{exit, write};

macro_rules! stub {
    // For functions that need to return a value (can't use -> !)
    ($name:ident, $ret:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() -> $ret {
            let msg = concat!("STUB: ", stringify!($name), "\n");
            write(2, msg.as_ptr(), msg.len());
            exit(99)
        }
    };
    // For variadic or complex signatures - just define the symbol
    ($name:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() -> ! {
            let msg = concat!("STUB: ", stringify!($name), "\n");
            write(2, msg.as_ptr(), msg.len());
            exit(99)
        }
    };
}

// Entry point - _start calls __libc_start_main
// This is what the system crt0 normally provides
unsafe extern "C" {
    fn main(argc: i32, argv: *const *const u8, envp: *const *const u8) -> i32;
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    // Kernel enters with RSP aligned to 16 bytes, pointing to argc
    // We need to call __libc_start_main(main, argc, argv, init, fini, rtld_fini, stack_end)
    core::arch::naked_asm!(
        "xor rbp, rbp",              // Clear frame pointer (ABI requirement)
        "lea rdi, [{main}]",         // arg1: main function address
        "mov rsi, [rsp]",            // arg2: argc
        "lea rdx, [rsp + 8]",        // arg3: argv
        "xor rcx, rcx",              // arg4: init (NULL)
        "xor r8, r8",                // arg5: fini (NULL)
        "xor r9, r9",                // arg6: rtld_fini (NULL)
        "push rsp",                  // arg7: stack_end (on stack)
        "and rsp, -16",              // Align stack to 16 bytes
        "call __libc_start_main",
        "ud2",
        main = sym main,
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn __libc_start_main(
    main: extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    argc: i32,
    argv: *const *const u8,
    _init: extern "C" fn(),
    _fini: extern "C" fn(),
    _rtld_fini: extern "C" fn(),
    _stack_end: *mut u8,
) -> ! {
    crate::init_heap();
    let envp = unsafe { argv.offset(argc as isize + 1) };
    let ret = main(argc, argv, envp);
    exit(ret)
}

// Memory
stub!(abort);
stub!(bcmp, i32);
// calloc - already implemented in lib.rs
// free - already implemented in lib.rs
// malloc - already implemented in lib.rs
// realloc - already implemented in lib.rs
stub!(posix_memalign, i32);
stub!(memcpy, *mut u8);
stub!(memmove, *mut u8);
stub!(memset, *mut u8);

// Memory mapping
stub!(mmap64, *mut u8);
stub!(mprotect, i32);
stub!(munmap, i32);

// File I/O
stub!(close, i32);
stub!(dup, i32);
stub!(fcntl, i32);
stub!(fstat64, i32);
stub!(lseek64, i64);
stub!(open64, i32);
// read - already implemented in lib.rs
stub!(stat64, i32);
// write - already implemented in lib.rs
stub!(writev, isize);

// File system
stub!(getcwd, *mut u8);
stub!(readlink, isize);
stub!(realpath, *mut u8);

// Environment
stub!(getenv, *const u8);
stub!(getauxval, usize);

// Error handling
#[unsafe(no_mangle)]
pub extern "C" fn __errno_location() -> *mut i32 {
    static mut ERRNO: i32 = 0;
    core::ptr::addr_of_mut!(ERRNO)
}

stub!(__xpg_strerror_r, i32);
stub!(strlen, usize);

// Signals - minimal implementations that just succeed
const SIG_DFL: usize = 0;

#[unsafe(no_mangle)]
pub extern "C" fn signal(_signum: i32, _handler: usize) -> usize {
    // Return SIG_DFL (previous handler) - pretend we set it
    SIG_DFL
}

#[unsafe(no_mangle)]
pub extern "C" fn sigaction(_signum: i32, _act: *const u8, _oldact: *mut u8) -> i32 {
    // Return success without doing anything
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn sigaltstack(_ss: *const u8, _old_ss: *mut u8) -> i32 {
    // Return success without doing anything
    0
}

stub!(pause, i32);

// poll - actually implement this one since std needs it at startup
const SYS_POLL: u64 = 7;

#[repr(C)]
pub struct pollfd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

#[unsafe(no_mangle)]
pub extern "C" fn poll(fds: *mut pollfd, nfds: u64, timeout: i32) -> i32 {
    unsafe { crate::syscall3(SYS_POLL, fds as u64, nfds, timeout as u64) as i32 }
}

// Threading - minimal single-threaded implementations
#[unsafe(no_mangle)]
pub extern "C" fn pthread_self() -> usize {
    // Return a constant "thread id" for the main thread
    1
}

// pthread_attr functions - used by std to get stack information
#[unsafe(no_mangle)]
pub extern "C" fn pthread_getattr_np(_thread: usize, _attr: *mut u8) -> i32 {
    // Return success - attr will be queried by getstack/getguardsize
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_attr_getstack(
    _attr: *const u8,
    stackaddr: *mut *mut u8,
    stacksize: *mut usize,
) -> i32 {
    // Provide fake but reasonable stack info
    unsafe {
        *stackaddr = 0x700000000000 as *mut u8; // Fake stack base
        *stacksize = 8 * 1024 * 1024; // 8MB stack
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_attr_getguardsize(_attr: *const u8, guardsize: *mut usize) -> i32 {
    unsafe {
        *guardsize = 4096; // One page guard
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_attr_destroy(_attr: *mut u8) -> i32 {
    0 // Nothing to destroy
}

stub!(pthread_key_create, i32);
stub!(pthread_key_delete, i32);
stub!(pthread_setspecific, i32);

// TLS
stub!(__tls_get_addr, *mut u8);

// System
stub!(syscall, i64);

// sysconf - return system configuration values
const _SC_PAGESIZE: i32 = 30;
const _SC_NPROCESSORS_ONLN: i32 = 84;

#[unsafe(no_mangle)]
pub extern "C" fn sysconf(name: i32) -> i64 {
    match name {
        _SC_PAGESIZE => 4096,
        _SC_NPROCESSORS_ONLN => 1,
        _ => -1, // Unknown, return error
    }
}

// Unwinding (for panic/backtrace)
stub!(_Unwind_Backtrace, i32);
stub!(_Unwind_DeleteException);
stub!(_Unwind_GetDataRelBase, usize);
stub!(_Unwind_GetIP, usize);
stub!(_Unwind_GetIPInfo, usize);
stub!(_Unwind_GetLanguageSpecificData, *mut u8);
stub!(_Unwind_GetRegionStart, usize);
stub!(_Unwind_GetTextRelBase, usize);
stub!(_Unwind_RaiseException, i32);
stub!(_Unwind_Resume);
stub!(_Unwind_SetGR);
stub!(_Unwind_SetIP);

// Dynamic linking
stub!(dl_iterate_phdr, i32);
