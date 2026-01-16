//! Environment variable support
//!
//! Provides the `environ` symbol and `getenv` function needed by std::env.

use core::ffi::{CStr, c_char};

/// Global environment pointer - set during startup by __libc_start_main
#[unsafe(no_mangle)]
pub static mut environ: *const *const c_char = core::ptr::null();

/// Initialize environ from the environment pointer passed to __libc_start_main
pub fn init(envp: *const *const c_char) {
    unsafe {
        environ = envp;
    }
}

/// Iterator over environment entries
struct EnvIter(*const *const c_char);

impl Iterator for EnvIter {
    type Item = &'static CStr;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.0.is_null() || (*self.0).is_null() {
                return None;
            }
            let entry = CStr::from_ptr(*self.0);
            self.0 = self.0.add(1);
            Some(entry)
        }
    }
}

/// Safe iterator over environ entries
fn iter_environ() -> EnvIter {
    EnvIter(unsafe { environ })
}

/// Get an environment variable by name.
/// Returns pointer to the value (after '=') or null if not found.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getenv(name: *const c_char) -> *const c_char {
    if name.is_null() {
        return core::ptr::null();
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_bytes = name_cstr.to_bytes();

    if name_bytes.is_empty() {
        return core::ptr::null();
    }

    for entry in iter_environ() {
        let entry_bytes = entry.to_bytes();
        if let Some(rest) = entry_bytes.strip_prefix(name_bytes)
            && rest.first() == Some(&b'=')
        {
            // Return pointer to value (after '=')
            return rest[1..].as_ptr() as *const c_char;
        }
    }

    core::ptr::null()
}
