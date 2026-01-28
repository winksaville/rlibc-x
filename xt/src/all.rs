//! Batch operations on all workspace crates
//!
//! Provides build_all, run_all, test_all for operating on all workspace members.

use std::os::unix::fs::PermissionsExt;
use std::process::ExitCode;

use crate::binary::{binary_size, strip_binary};
use crate::build::build_crate;
use crate::run::run_binary;
use crate::testing::test_crate;
use crate::workspace::{CrateKind, WorkspaceInfo};
use crate::{print_header, print_hline};

/// Result of a batch operation on a single crate
pub struct OpResult {
    pub name: String,
    pub success: bool,
    pub message: String,
    pub size: Option<u64>,
}

/// Build all workspace crates (excluding build tools)
pub fn build_all(
    workspace: &WorkspaceInfo,
    tspec: Option<&str>,
    release: bool,
    strip: bool,
    fail_fast: bool,
) -> Vec<OpResult> {
    let mut results = Vec::new();

    for member in workspace.buildable_members() {
        println!("=== {} ===", member.name);

        let result = match build_crate(&member.name, tspec, release) {
            Ok(build_result) => {
                if strip && member.has_binary {
                    if let Err(e) = strip_binary(&build_result.binary_path) {
                        eprintln!("  warning: strip failed: {}", e);
                    }
                }
                let size = binary_size(&build_result.binary_path).ok();
                OpResult {
                    name: member.name.clone(),
                    success: true,
                    message: format!("{}", build_result.binary_path.display()),
                    size,
                }
            }
            Err(e) => OpResult {
                name: member.name.clone(),
                success: false,
                message: e.to_string(),
                size: None,
            },
        };

        let failed = !result.success;
        results.push(result);

        if failed && fail_fast {
            break;
        }
    }

    results
}

/// Run all app crates sequentially
pub fn run_all(
    workspace: &WorkspaceInfo,
    tspec: Option<&str>,
    release: bool,
    strip: bool,
) -> Vec<OpResult> {
    let mut results = Vec::new();

    for member in workspace.runnable_members() {
        println!("=== {} ===", member.name);

        let result = match build_crate(&member.name, tspec, release) {
            Ok(build_result) => {
                if strip {
                    if let Err(e) = strip_binary(&build_result.binary_path) {
                        eprintln!("  warning: strip failed: {}", e);
                    }
                }
                match run_binary(&build_result.binary_path) {
                    Ok(exit_code) => OpResult {
                        name: member.name.clone(),
                        success: true, // We don't treat non-zero exit as failure
                        message: format!("exit code: {}", exit_code),
                        size: None,
                    },
                    Err(e) => OpResult {
                        name: member.name.clone(),
                        success: false,
                        message: format!("run failed: {}", e),
                        size: None,
                    },
                }
            }
            Err(e) => OpResult {
                name: member.name.clone(),
                success: false,
                message: format!("build failed: {}", e),
                size: None,
            },
        };

        results.push(result);
    }

    results
}

/// Test all workspace crates
pub fn test_all(
    workspace: &WorkspaceInfo,
    tspec: Option<&str>,
    release: bool,
    fail_fast: bool,
) -> Vec<OpResult> {
    let mut results = Vec::new();

    // Test regular crates (excluding Test kind which needs special handling)
    for member in workspace.buildable_members() {
        if member.kind == CrateKind::Test {
            continue; // Handle test crates separately
        }

        println!("=== {} ===", member.name);

        let result = match test_crate(&member.name, tspec, release) {
            Ok(()) => OpResult {
                name: member.name.clone(),
                success: true,
                message: "ok".to_string(),
                size: None,
            },
            Err(e) => OpResult {
                name: member.name.clone(),
                success: false,
                message: e.to_string(),
                size: None,
            },
        };

        let failed = !result.success;
        results.push(result);

        if failed && fail_fast {
            return results;
        }
    }

    // Handle test crates (like rlibc-x2-tests) - build and run all test binaries
    for member in workspace.test_members() {
        println!("=== {} ===", member.name);

        // Build the test crate (builds all binaries)
        if let Err(e) = build_crate(&member.name, tspec, release) {
            results.push(OpResult {
                name: member.name.clone(),
                success: false,
                message: format!("build failed: {}", e),
                size: None,
            });
            if fail_fast {
                return results;
            }
            continue;
        }

        // Find and run all test binaries in the target directory
        let profile = if release { "release" } else { "debug" };
        let target_dir = workspace.root.join("target").join(profile);

        // Look for binaries that end with "-tests" in the target directory
        let test_binaries: Vec<_> = std::fs::read_dir(&target_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let is_file = e.path().is_file();
                let is_test = name.ends_with("-tests");
                let is_executable = e
                    .metadata()
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false);
                is_file && is_test && is_executable
            })
            .collect();

        if test_binaries.is_empty() {
            results.push(OpResult {
                name: member.name.clone(),
                success: false,
                message: "no test binaries found".to_string(),
                size: None,
            });
            continue;
        }

        for entry in test_binaries {
            let path = entry.path();
            let bin_name = entry.file_name().to_string_lossy().to_string();

            print!("  Running {}... ", bin_name);

            let result = match run_binary(&path) {
                Ok(exit_code) => {
                    if exit_code == 0 {
                        println!("ok");
                        OpResult {
                            name: format!("{}/{}", member.name, bin_name),
                            success: true,
                            message: "ok".to_string(),
                            size: None,
                        }
                    } else {
                        println!("FAILED (exit {})", exit_code);
                        OpResult {
                            name: format!("{}/{}", member.name, bin_name),
                            success: false,
                            message: format!("exit code: {}", exit_code),
                            size: None,
                        }
                    }
                }
                Err(e) => {
                    println!("FAILED");
                    OpResult {
                        name: format!("{}/{}", member.name, bin_name),
                        success: false,
                        message: format!("run failed: {}", e),
                        size: None,
                    }
                }
            };

            let failed = !result.success;
            results.push(result);

            if failed && fail_fast {
                return results;
            }
        }
    }

    results
}

/// Print a summary of operation results (for tests)
pub fn print_test_summary(results: &[OpResult]) -> ExitCode {
    let max_name_len = results
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or(5)
        .max(5);

    println!();
    print_header!("TEST SUMMARY");
    println!("  {:width$}  Status", "Crate", width = max_name_len);

    let mut passed = 0;
    let mut failed = 0;

    for result in results {
        let status = if result.success {
            passed += 1;
            "[PASS]"
        } else {
            failed += 1;
            "[FAIL]"
        };
        println!("  {:width$}  {status}", result.name, width = max_name_len);
    }

    println!();
    println!("  Test: {} passed, {} failed", passed, failed);
    println!();
    println!("  Note: Run `cargo xt test xt` or `cargo test -p xt` to test xt itself");
    print_hline!();
    println!();

    if failed > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// Print a summary for build operations (OK/FAILED)
pub fn print_summary(results: &[OpResult]) -> ExitCode {
    let max_name_len = results
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or(5)
        .max(5);

    println!();
    print_header!("BUILD SUMMARY");
    println!("  {:width$}  Status    Size", "Crate", width = max_name_len);

    let mut ok_count = 0;
    let mut failed_count = 0;

    for result in results {
        let status = if result.success {
            ok_count += 1;
            "[ OK ]"
        } else {
            failed_count += 1;
            "[FAIL]"
        };
        let size_str = result
            .size
            .map(format_size)
            .unwrap_or_else(|| "--".to_string());
        println!(
            "  {:width$}  {status}  {:>6}",
            result.name,
            size_str,
            width = max_name_len
        );
    }

    println!();
    println!("  Build: {} ok, {} failed", ok_count, failed_count);
    print_hline!();
    println!();

    if failed_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1}M", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1}K", bytes as f64 / 1_000.0)
    } else {
        format!("{}", bytes)
    }
}

/// Print a summary for run operations (shows exit codes, not pass/fail)
pub fn print_run_summary(results: &[OpResult]) -> ExitCode {
    let max_name_len = results
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or(5)
        .max(5);

    println!();
    print_header!("RUN SUMMARY");
    println!("  {:width$}  Exit", "Crate", width = max_name_len);

    let mut error_count = 0;

    for result in results {
        if result.success {
            // Extract exit code number from message "exit code: X"
            let code = result
                .message
                .strip_prefix("exit code: ")
                .unwrap_or(&result.message);
            println!(
                "  {:width$}  {:>4}",
                result.name,
                code,
                width = max_name_len
            );
        } else {
            error_count += 1;
            println!(
                "  {:width$}  ERROR: {}",
                result.name,
                result.message,
                width = max_name_len
            );
        }
    }

    println!();
    if error_count > 0 {
        println!("  Run: {} error(s)", error_count);
    }
    print_hline!();
    println!();

    if error_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
