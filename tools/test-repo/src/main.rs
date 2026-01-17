//! Run all repository tests
//!
//! This tool runs:
//! 1. `cargo test` - default target tests
//! 2. `cargo test --target x86_64-unknown-linux-musl` - musl-specific tests
//! 3. rlibc-x2-tests binaries - standalone integration tests
//!
//! # Usage
//!
//! ```text
//! cargo run -p test-repo
//! cargo run -p test-repo -- -v          # verbose output
//! cargo run -p test-repo -- --fail-fast # stop on first failure
//! cargo run -p test-repo -- --release   # use release builds (default)
//! cargo run -p test-repo -- --debug     # use debug builds (faster iteration)
//! ```

use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

struct Config {
    verbose: bool,
    fail_fast: bool,
    debug: bool,
}

struct TestResult {
    name: String,
    passed: bool,
    #[allow(dead_code)] // Captured for potential future use (e.g., detailed failure reports)
    output: String,
}

fn main() -> ExitCode {
    let config = parse_args();
    let mut results = Vec::new();

    // Get workspace root (we're in tools/test-repo)
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("Could not find workspace root");

    // 1. cargo test (default target)
    print_section("cargo test (default target)");
    let result = run_cargo_test("cargo test", &[], workspace_root, &config);
    let failed = !result.passed;
    results.push(result);
    if failed && config.fail_fast {
        return print_summary(&results);
    }

    // 2. cargo test (musl target)
    print_section("cargo test (musl target)");
    let result = run_cargo_test(
        "cargo test (musl)",
        &[
            "--target",
            "x86_64-unknown-linux-musl",
            "-p",
            "ex-musl",
            "-p",
            "hw-musl",
        ],
        workspace_root,
        &config,
    );
    let failed = !result.passed;
    results.push(result);
    if failed && config.fail_fast {
        return print_summary(&results);
    }

    // 3. rlibc-x2-tests
    print_section("rlibc-x2-tests");
    let rlibc_x2_results = run_rlibc_x2_tests(workspace_root, &config);
    for result in rlibc_x2_results {
        let failed = !result.passed;
        results.push(result);
        if failed && config.fail_fast {
            return print_summary(&results);
        }
    }

    print_summary(&results)
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut config = Config {
        verbose: false,
        fail_fast: false,
        debug: false,
    };

    for arg in args {
        match arg.as_str() {
            "-v" | "--verbose" => config.verbose = true,
            "-f" | "--fail-fast" | "--fail" => config.fail_fast = true,
            "-r" | "--release" => config.debug = false,
            "-d" | "--debug" => config.debug = true,
            "-h" | "--help" => {
                println!("Usage: test-repo [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -v, --verbose     Show full test output");
                println!("  -f, --fail-fast   Stop on first failure");
                println!("  -r, --release     Use release builds (default)");
                println!("  -d, --debug       Use debug builds (faster iteration)");
                println!("  -h, --help        Show this help");
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown option: {other}");
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    config
}

fn print_section(name: &str) {
    println!("=== {name} ===");
}

fn run_cargo_test(
    name: &str,
    extra_args: &[&str],
    workspace_root: &Path,
    config: &Config,
) -> TestResult {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    if !config.debug {
        cmd.arg("--release");
    }
    cmd.args(extra_args);
    cmd.current_dir(workspace_root);

    if config.verbose {
        let status = cmd.status();
        let passed = matches!(status, Ok(s) if s.success());
        println!();
        TestResult {
            name: name.to_string(),
            passed,
            output: String::new(),
        }
    } else {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{stderr}\n{stdout}");
                let passed = output.status.success();

                // Collect crate names from stderr ("Running" lines)
                let crate_names: Vec<String> = stderr
                    .lines()
                    .filter(|line| line.trim_start().starts_with("Running "))
                    .filter_map(|line| {
                        line.find("deps/").and_then(|pos| {
                            let rest = &line[pos + 5..];
                            rest.find('-').map(|dash| rest[..dash].replace('_', "-"))
                        })
                    })
                    .collect();

                // Parse test results from stdout
                let mut total_passed = 0;
                let mut total_failed = 0;
                let mut total_ignored = 0;
                let mut crate_idx = 0;

                for line in stdout.lines() {
                    if line.contains("test result:") {
                        let (p, f, i) = parse_test_result(line);
                        let crate_name = crate_names
                            .get(crate_idx)
                            .map(|s| s.as_str())
                            .unwrap_or("unknown");
                        crate_idx += 1;

                        if p > 0 || f > 0 || i > 0 {
                            let status = if f > 0 {
                                "FAIL"
                            } else if p > 0 {
                                "ok"
                            } else {
                                "skip"
                            };
                            println!(
                                "  {crate_name}: {status} ({p} passed, {f} failed, {i} ignored)"
                            );
                        }
                        total_passed += p;
                        total_failed += f;
                        total_ignored += i;
                    }
                }

                if passed {
                    println!(
                        "  Total: {total_passed} passed, {total_failed} failed, {total_ignored} ignored\n"
                    );
                } else {
                    println!(
                        "  FAILED: {total_passed} passed, {total_failed} failed, {total_ignored} ignored\n"
                    );
                    println!("{combined}");
                }

                TestResult {
                    name: name.to_string(),
                    passed,
                    output: combined.to_string(),
                }
            }
            Err(e) => {
                eprintln!("  Failed to run cargo test: {e}");
                TestResult {
                    name: name.to_string(),
                    passed: false,
                    output: e.to_string(),
                }
            }
        }
    }
}

fn parse_test_result(line: &str) -> (u32, u32, u32) {
    // Parse "test result: ok. X passed; Y failed; Z ignored; ..."
    let mut passed = 0;
    let mut failed = 0;
    let mut ignored = 0;

    // Extract numbers before "passed", "failed", "ignored"
    for part in line.split(';') {
        let part = part.trim();
        if part.ends_with(" passed") {
            // Find the number - it's the last word before "passed"
            let words: Vec<&str> = part.split_whitespace().collect();
            if words.len() >= 2
                && let Ok(n) = words[words.len() - 2].parse()
            {
                passed = n;
            }
        } else if part.ends_with(" failed") {
            let words: Vec<&str> = part.split_whitespace().collect();
            if words.len() >= 2
                && let Ok(n) = words[words.len() - 2].parse()
            {
                failed = n;
            }
        } else if part.ends_with(" ignored") {
            let words: Vec<&str> = part.split_whitespace().collect();
            if words.len() >= 2
                && let Ok(n) = words[words.len() - 2].parse()
            {
                ignored = n;
            }
        }
    }

    (passed, failed, ignored)
}

fn run_rlibc_x2_tests(workspace_root: &Path, config: &Config) -> Vec<TestResult> {
    let mut results = Vec::new();

    // First, build the test binaries
    print!("  Building rlibc-x2-tests... ");

    let mut build_cmd = Command::new("cargo");
    build_cmd.args(["build", "-p", "rlibc-x2-tests"]);
    if !config.debug {
        build_cmd.arg("--release");
    }
    let build_output = build_cmd
        .current_dir(workspace_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match build_output {
        Ok(output) if output.status.success() => {
            println!("ok");
        }
        Ok(output) => {
            println!("FAILED");
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("{stderr}");
            results.push(TestResult {
                name: "rlibc-x2-tests (build)".to_string(),
                passed: false,
                output: stderr.to_string(),
            });
            return results;
        }
        Err(e) => {
            println!("FAILED");
            eprintln!("  Failed to run cargo build: {e}");
            results.push(TestResult {
                name: "rlibc-x2-tests (build)".to_string(),
                passed: false,
                output: e.to_string(),
            });
            return results;
        }
    }

    // Find and run test binaries
    let profile = if config.debug { "debug" } else { "release" };
    let target_dir = workspace_root.join(format!("target/{profile}"));

    let test_binaries: Vec<_> = fs::read_dir(&target_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.ends_with("-tests") && e.path().is_file()
        })
        .collect();

    if test_binaries.is_empty() {
        eprintln!("  No test binaries found in {}", target_dir.display());
        results.push(TestResult {
            name: "rlibc-x2-tests (no binaries)".to_string(),
            passed: false,
            output: String::new(),
        });
        return results;
    }

    for entry in test_binaries {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        print!("  Running {name_str}... ");

        let output = Command::new(&path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{stdout}{stderr}");

                if passed {
                    println!("ok");
                    if config.verbose && !stdout.is_empty() {
                        println!("{stdout}");
                    }
                } else {
                    println!("FAILED");
                    println!("{combined}");
                }

                results.push(TestResult {
                    name: format!("rlibc-x2-tests/{name_str}"),
                    passed,
                    output: combined.to_string(),
                });
            }
            Err(e) => {
                println!("FAILED");
                let output = format!("Failed to run: {e}");
                eprintln!("  {output}");
                results.push(TestResult {
                    name: format!("rlibc-x2-tests/{name_str}"),
                    passed: false,
                    output,
                });
            }
        }
    }

    println!();
    results
}

fn print_summary(results: &[TestResult]) -> ExitCode {
    println!("========================================");
    println!("               SUMMARY");
    println!("========================================\n");

    let mut passed_count = 0;
    let mut failed_count = 0;

    for result in results {
        let status = if result.passed {
            passed_count += 1;
            "PASS"
        } else {
            failed_count += 1;
            "FAIL"
        };
        println!("  [{status}] {}", result.name);
    }

    println!(
        "\n  Total: {} passed, {} failed",
        passed_count, failed_count
    );
    println!("========================================\n");

    if failed_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
