//! Thread-local storage and pthread stubs

use crate::memory::malloc;
use crate::syscall::{syscall2, syscall3, ARCH_SET_FS, SYS_ARCH_PRCTL, SYS_POLL};

/// Initialize Thread-Local Storage
///
/// This sets up the FS segment register to point to a TLS block.
/// Must be called before any code that uses #[thread_local] statics.
pub fn init_tls() {
    // For now, allocate a minimal TLS block
    // The PT_TLS segment tells us the real size needed (24 bytes for our binary)
    // We allocate: [TLS data (24 bytes)] [self-pointer (8 bytes)]
    //                                     ^-- FS.base points here

    const TLS_SIZE: usize = 64; // Over-allocate to be safe
    const SELF_PTR_SIZE: usize = 8;

    unsafe {
        let block = malloc(TLS_SIZE + SELF_PTR_SIZE);
        if block.is_null() {
            // Can't do much without memory
            return;
        }

        // Zero-initialize the TLS block
        core::ptr::write_bytes(block, 0, TLS_SIZE + SELF_PTR_SIZE);

        // The self-pointer goes at the end, and FS.base points to it
        let tcb = block.add(TLS_SIZE) as *mut usize;
        *tcb = tcb as usize;

        // Set FS.base to point at the self-pointer (TCB)
        syscall2(SYS_ARCH_PRCTL, ARCH_SET_FS, tcb as u64);
    }
}

// --- pthread stubs ---

#[unsafe(no_mangle)]
pub extern "C" fn pthread_self() -> usize {
    1 // Return constant thread ID for main thread
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_getattr_np(_thread: usize, _attr: *mut u8) -> i32 {
    0 // Success
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_attr_getstack(
    _attr: *const u8,
    stackaddr: *mut *mut u8,
    stacksize: *mut usize,
) -> i32 {
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

#[unsafe(no_mangle)]
pub extern "C" fn pthread_key_create(
    _key: *mut u32,
    _destructor: Option<extern "C" fn(*mut u8)>,
) -> i32 {
    0 // Pretend success
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_key_delete(_key: u32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_setspecific(_key: u32, _value: *const u8) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_getspecific(_key: u32) -> *mut u8 {
    core::ptr::null_mut()
}

// --- TLS access stub ---

#[unsafe(no_mangle)]
pub extern "C" fn __tls_get_addr(_ti: *const u8) -> *mut u8 {
    // This shouldn't be called for static TLS, but provide a stub
    core::ptr::null_mut()
}

// --- poll (needed by std at startup) ---

#[repr(C)]
pub struct pollfd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

#[unsafe(no_mangle)]
pub extern "C" fn poll(fds: *mut pollfd, nfds: u64, timeout: i32) -> i32 {
    unsafe { syscall3(SYS_POLL, fds as u64, nfds, timeout as u64) as i32 }
}

// --- sysconf ---

const _SC_PAGESIZE: i32 = 30;
const _SC_NPROCESSORS_ONLN: i32 = 84;

#[unsafe(no_mangle)]
pub extern "C" fn sysconf(name: i32) -> i64 {
    match name {
        _SC_PAGESIZE => 4096,
        _SC_NPROCESSORS_ONLN => 1,
        _ => -1,
    }
}
