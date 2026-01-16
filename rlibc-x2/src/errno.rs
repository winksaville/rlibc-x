//! errno support

static mut ERRNO: i32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn __errno_location() -> *mut i32 {
    core::ptr::addr_of_mut!(ERRNO)
}

#[unsafe(no_mangle)]
pub extern "C" fn __xpg_strerror_r(_errnum: i32, _buf: *mut u8, _buflen: usize) -> i32 {
    // TODO: implement properly
    -1
}

/// Get the length of a null-terminated string.
///
/// # Safety
/// `s` must be a valid null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}
