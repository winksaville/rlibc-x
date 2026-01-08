//! Test: environ support
//!
//! Exit codes:
//!   0 = all tests passed
//!   1 = environ is null
//!   2 = environ has no entries
//!   3 = PATH not found via raw environ
//!   4 = std::env::var("PATH") failed

use std::ffi::CStr;
use std::process::ExitCode;

extern crate rlibc_x2;

fn main() -> ExitCode {
    // Test 1: environ should not be null
    let env_ptr = unsafe { rlibc_x2::environ };
    if env_ptr.is_null() {
        eprintln!("FAIL: environ is null");
        return ExitCode::from(1);
    }

    // Test 2: environ should have at least one entry
    let first_entry = unsafe { *env_ptr };
    if first_entry.is_null() {
        eprintln!("FAIL: environ has no entries");
        return ExitCode::from(2);
    }

    // Test 3: PATH should be present via raw environ
    let mut found_path = false;
    let mut current = env_ptr;
    unsafe {
        while !(*current).is_null() {
            let entry = CStr::from_ptr(*current);
            if let Ok(s) = entry.to_str() {
                if s.starts_with("PATH=") {
                    found_path = true;
                    break;
                }
            }
            current = current.add(1);
        }
    }

    if !found_path {
        eprintln!("FAIL: PATH not found in environ");
        return ExitCode::from(3);
    }

    // Test 4: std::env::var() works
    match std::env::var("PATH") {
        Ok(path) => {
            if std::env::var("VERBOSE").is_ok() {
                eprintln!("PATH={}", path);
            }
        }
        Err(e) => {
            eprintln!("FAIL: std::env::var(\"PATH\") failed: {}", e);
            return ExitCode::from(4);
        }
    }

    eprintln!("PASS: environ");
    ExitCode::from(0)
}
