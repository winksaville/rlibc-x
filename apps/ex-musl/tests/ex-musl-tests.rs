//! Test that ex-musl binary does not dynamically link libc.
//! Note: musl is statically linked, so is_libc_used reports no libc dependency.
//! This test only runs when built with musl target (cargo t-musl).

use is_libc_used::is_libc_used;
use std::path::Path;

#[test]
#[cfg_attr(not(target_env = "musl"), ignore)]
fn binary_does_not_use_libc() {
    let binary = env!("CARGO_BIN_EXE_ex-musl");
    let result = is_libc_used(Path::new(binary)).expect("should parse binary");
    assert!(
        !result.uses_libc,
        "ex-musl should NOT dynamically link libc (musl is static): {:?}",
        result.info
    );
}
