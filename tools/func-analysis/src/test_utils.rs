//! Shared test utilities for func-analysis
//!
//! Uses a standalone test binary (test-fixtures/test_binary.rs) compiled
//! directly with rustc. This avoids dependencies on workspace crates and
//! the xt build system, making tests self-contained.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

static TEST_BINARY_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Build the standalone test binary and return its path.
///
/// The binary is compiled once per test run using rustc directly.
/// Source: test-fixtures/test_binary.rs
pub fn build_test_binary() -> PathBuf {
    TEST_BINARY_PATH
        .get_or_init(|| {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let source = manifest_dir.join("test-fixtures/test_binary.rs");
            let output_dir = manifest_dir.join("target/test-fixtures");
            let output = output_dir.join("test_binary");

            // Create output directory
            std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");

            // Compile with rustc in release mode
            let result = Command::new("rustc")
                .args([
                    "-O", // Optimize (release mode)
                    "-o",
                    output.to_str().unwrap(),
                    source.to_str().unwrap(),
                ])
                .output()
                .expect("Failed to run rustc");

            assert!(
                result.status.success(),
                "Failed to compile test binary: {}",
                String::from_utf8_lossy(&result.stderr)
            );

            output
        })
        .clone()
}
