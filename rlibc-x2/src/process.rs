//! Process management (_start, exit, __libc_start_main)

use crate::memory::init_heap;
use crate::syscall::{SYS_EXIT, syscall1};
use crate::thread::init_tls;

/// Exit the process with the given code
pub fn exit(code: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code as u64);
    }
    loop {}
}

/// C-compatible abort
#[unsafe(no_mangle)]
pub extern "C" fn abort() -> ! {
    exit(134) // 128 + SIGABRT(6)
}

// External reference to the C ABI main that rustc generates
unsafe extern "C" {
    fn main(argc: i32, argv: *const *const u8, envp: *const *const u8) -> i32;
}

/// The libc entry point called by _start
#[unsafe(no_mangle)]
pub extern "C" fn __libc_start_main(
    main_fn: extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    argc: i32,
    argv: *const *const u8,
    _init: usize,
    _fini: usize,
    _rtld_fini: usize,
    _stack_end: *mut u8,
) -> ! {
    // Initialize heap first (TLS needs malloc)
    init_heap();

    // Initialize TLS before calling main
    init_tls();

    // Calculate envp (follows argv + NULL terminator)
    let envp = unsafe { argv.offset(argc as isize + 1) };

    // Call main and exit with its return value
    let ret = main_fn(argc, argv, envp);
    exit(ret)
}

/// Program entry point - called by the kernel
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    // Kernel enters with RSP pointing to argc on the stack
    // Stack layout: [argc] [argv[0]] [argv[1]] ... [NULL] [envp[0]] ...
    //
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
        "ud2",                       // Should never return
        main = sym main,
    );
}
