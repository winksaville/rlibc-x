//! rlibc-x2: A libc replacement for Rust std applications
//!
//! This crate provides the libc symbols needed to run Rust programs
//! that use the standard library, without linking to glibc.
//!
//! # Usage
//!
//! Add to your Cargo.toml:
//! ```toml
//! [dependencies]
//! rlibc-x2 = { path = "../rlibc-x2" }
//! ```
//!
//! And in your build.rs:
//! ```rust
//! fn main() {
//!     println!("cargo:rustc-link-arg=-static");
//!     println!("cargo:rustc-link-arg=-nostdlib");
//!     println!("cargo:rustc-link-arg=-nodefaultlibs");
//!     println!("cargo:rustc-link-arg=-e_start");
//!     println!("cargo:rustc-link-arg=-Wl,--undefined=_start");
//!     println!("cargo:rustc-link-arg=-Wl,--undefined=__libc_start_main");
//! }
//! ```

#![no_std]

pub mod errno;
pub mod io;
pub mod memory;
pub mod process;
pub mod signal;
pub mod syscall;
pub mod thread;

// Re-export key functions
pub use process::exit;

// Note: We don't provide a panic handler because std provides one.
// rlibc-x2 is designed to be used WITH std, just replacing libc.

// Stubs for functions we haven't implemented yet
// These print a message and exit so we know what's missing

macro_rules! stub {
    ($name:ident, $ret:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() -> $ret {
            let msg = concat!("STUB: ", stringify!($name), "\n");
            io::write(2, msg.as_ptr(), msg.len());
            process::exit(99)
        }
    };
    ($name:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() -> ! {
            let msg = concat!("STUB: ", stringify!($name), "\n");
            io::write(2, msg.as_ptr(), msg.len());
            process::exit(99)
        }
    };
}

// File system stubs
stub!(close, i32);
stub!(dup, i32);
stub!(fcntl, i32);
stub!(fstat64, i32);
stub!(lseek64, i64);
stub!(open64, i32);
stub!(stat64, i32);
stub!(getcwd, *mut u8);
stub!(readlink, isize);
stub!(realpath, *mut u8);

// Environment stubs
stub!(getenv, *const u8);
stub!(getauxval, usize);

// Memory mapping stubs
stub!(mmap64, *mut u8);
stub!(mprotect, i32);
stub!(munmap, i32);

// System stubs
stub!(syscall, i64);

// Unwinding stubs (for panic/backtrace)
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

// Dynamic linking stubs
stub!(dl_iterate_phdr, i32);
