//! Integration tests for is-libc-used library.

use is_libc_used::is_libc_used;
use std::path::Path;

#[test]
fn usr_bin_ls_uses_libc() {
    let path = Path::new("/usr/bin/ls");
    if !path.exists() {
        // Skip test if /usr/bin/ls doesn't exist
        return;
    }

    let result = is_libc_used(path).expect("should parse /usr/bin/ls");
    assert!(
        result.uses_libc,
        "/usr/bin/ls should use libc: {:?}",
        result.info
    );
}

#[test]
fn ex_x1_no_libc() {
    // Find ex-x1 binary relative to the workspace root
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("should find workspace root");

    // Try debug first, then release
    let debug_path = workspace_root.join("target/debug/ex-x1");
    let release_path = workspace_root.join("target/release/ex-x1");

    let path = if debug_path.exists() {
        debug_path
    } else if release_path.exists() {
        release_path
    } else {
        // Skip test if ex-x1 not built
        eprintln!("Skipping: ex-x1 not found (run 'cargo build -p ex-x1' first)");
        return;
    };

    let result = is_libc_used(&path).expect("should parse ex-x1");
    assert!(
        !result.uses_libc,
        "ex-x1 should NOT use libc: {:?}",
        result.info
    );
}
