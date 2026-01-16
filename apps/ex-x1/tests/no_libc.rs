//! Test that ex-x1 binary does not use libc.

use is_libc_used::is_libc_used;
use std::path::Path;

#[test]
fn binary_does_not_use_libc() {
    let binary = env!("CARGO_BIN_EXE_ex-x1");
    let result = is_libc_used(Path::new(binary)).expect("should parse binary");
    assert!(
        !result.uses_libc,
        "ex-x1 should NOT use libc: {:?}",
        result.info
    );
}
