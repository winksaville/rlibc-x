//! Test that hw-glibc binary DOES use libc.

use is_libc_used::is_libc_used;
use std::path::Path;

#[test]
fn binary_uses_libc() {
    let binary = env!("CARGO_BIN_EXE_hw-glibc");
    let result = is_libc_used(Path::new(binary)).expect("should parse binary");
    assert!(
        result.uses_libc,
        "hw-glibc SHOULD use libc: {:?}",
        result.info
    );
}
