//! Test: std::env API integration
//!
//! Tests that Rust's safe std::env API works correctly with our
//! environ/getenv implementation.
//!
//! Exit codes:
//!   0 = all tests passed
//!   1 = std::env::var() failed for PATH
//!   2 = std::env::var_os() failed for PATH
//!   3 = std::env::vars() returned no entries
//!   4 = PATH not found via vars() iteration
//!   5 = var() and vars() returned different PATH values

use std::env;
use std::process::ExitCode;

extern crate rlibc_x2;

fn main() -> ExitCode {
    let verbose = env::var("VERBOSE").is_ok();

    // Test 1: std::env::var() works
    let path_from_var = match env::var("PATH") {
        Ok(path) => {
            if verbose {
                eprintln!("env::var(\"PATH\") = {}", path);
            }
            path
        }
        Err(e) => {
            eprintln!("FAIL: env::var(\"PATH\"): {}", e);
            return ExitCode::from(1);
        }
    };

    // Test 2: std::env::var_os() works
    match env::var_os("PATH") {
        Some(path) => {
            if verbose {
                eprintln!("env::var_os(\"PATH\") = {:?}", path);
            }
        }
        None => {
            eprintln!("FAIL: env::var_os(\"PATH\") returned None");
            return ExitCode::from(2);
        }
    }

    // Test 3: std::env::vars() returns entries
    let all_vars: Vec<(String, String)> = env::vars().collect();
    if all_vars.is_empty() {
        eprintln!("FAIL: env::vars() returned no entries");
        return ExitCode::from(3);
    }
    if verbose {
        eprintln!("env::vars() returned {} entries", all_vars.len());
    }

    // Test 4: PATH is findable via vars() iteration
    let path_from_iter = all_vars
        .iter()
        .find(|(k, _)| k == "PATH")
        .map(|(_, v)| v.clone());

    let path_from_iter = match path_from_iter {
        Some(path) => path,
        None => {
            eprintln!("FAIL: PATH not found via env::vars() iteration");
            return ExitCode::from(4);
        }
    };

    // Test 5: Both methods return the same value
    if path_from_var != path_from_iter {
        eprintln!("FAIL: var() and vars() returned different PATH values");
        eprintln!("  var():  {}", path_from_var);
        eprintln!("  vars(): {}", path_from_iter);
        return ExitCode::from(5);
    }

    // Test 6: Non-existent variable returns appropriate error/None
    if env::var("RLIBC_X_NONEXISTENT_VAR_12345").is_ok() {
        eprintln!("WARN: unexpected variable RLIBC_X_NONEXISTENT_VAR_12345 exists");
        // Not a failure, just unexpected
    }

    if env::var_os("RLIBC_X_NONEXISTENT_VAR_12345").is_some() {
        eprintln!("WARN: unexpected variable RLIBC_X_NONEXISTENT_VAR_12345 exists");
    }

    eprintln!("PASS: std_env ({} variables)", all_vars.len());
    ExitCode::from(0)
}
