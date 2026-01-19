//! Shared test utilities for func-analysis

use std::path::PathBuf;
use std::process::Command;

/// Build ex-x2 in release mode and return its path
pub fn build_test_binary() -> PathBuf {
    // Get workspace root (two levels up from tools/func-analysis)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

    // Build ex-x2 in release mode from workspace root
    let status = Command::new("cargo")
        .args(["build", "-p", "ex-x2", "--release"])
        .current_dir(workspace_root)
        .status()
        .expect("Failed to run cargo build");
    assert!(status.success(), "Failed to build ex-x2");

    workspace_root.join("target/release/ex-x2")
}
