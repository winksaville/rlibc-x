//! Signal handling stubs

const SIG_DFL: usize = 0;

#[unsafe(no_mangle)]
pub extern "C" fn signal(_signum: i32, _handler: usize) -> usize {
    SIG_DFL // Return previous handler (pretend it was default)
}

#[unsafe(no_mangle)]
pub extern "C" fn sigaction(_signum: i32, _act: *const u8, _oldact: *mut u8) -> i32 {
    0 // Success
}

#[unsafe(no_mangle)]
pub extern "C" fn sigaltstack(_ss: *const u8, _old_ss: *mut u8) -> i32 {
    0 // Success
}

#[unsafe(no_mangle)]
pub extern "C" fn pause() -> i32 {
    -1 // Always returns -1 with EINTR
}
