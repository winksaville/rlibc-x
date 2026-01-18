//! Project automation tasks
//!
//! # Usage
//!
//! ```text
//! cargo xtask test                  # run all tests (release builds, default)
//! cargo xtask test .                # test crate in current directory
//! cargo xtask test ex-x1            # test specific crate
//! cargo xtask test ex-x1 hw-x2      # test multiple crates
//! cargo xtask test ex-musl          # auto-detects musl target
//! cargo xtask test rlibc-x2         # includes rlibc-x2-tests binaries
//! cargo xtask test -v               # verbose output
//! cargo xtask test --debug          # use debug builds (faster iteration)
//! ```
//!
//! # Available Commands
//!
//! - `test` - Run repository tests (all, or specific crates)

use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

enum Subcommand {
    Test,
}

struct Config {
    subcommand: Subcommand,
    verbose: bool,
    fail_fast: bool,
    debug: bool,
    crates: Vec<String>, // Empty means all crates
}

struct TestResult {
    name: String,
    passed: bool,
    #[allow(dead_code)] // Captured for potential future use (e.g., detailed failure reports)
    output: String,
}

fn main() -> ExitCode {
    let config = parse_args();

    // Get workspace root (xtask is at workspace root level)
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Could not find workspace root");

    match config.subcommand {
        Subcommand::Test => run_tests(&config, workspace_root),
    }
}

fn run_tests(config: &Config, workspace_root: &Path) -> ExitCode {
    let mut results = Vec::new();

    if config.crates.is_empty() {
        // Run all tests (original behavior)
        results.extend(run_all_tests(config, workspace_root));
    } else {
        // Run tests for specified crates only
        results.extend(run_filtered_tests(config, workspace_root));
    }

    print_summary(&results)
}

fn run_all_tests(config: &Config, workspace_root: &Path) -> Vec<TestResult> {
    let mut results = Vec::new();

    // 1. cargo test (default target)
    print_section("cargo test (default target)");
    let result = run_cargo_test("cargo test", &[], workspace_root, config);
    let failed = !result.passed;
    results.push(result);
    if failed && config.fail_fast {
        return results;
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
        config,
    );
    let failed = !result.passed;
    results.push(result);
    if failed && config.fail_fast {
        return results;
    }

    // 3. rlibc-x2-tests
    print_section("rlibc-x2-tests");
    let rlibc_x2_results = run_rlibc_x2_tests(workspace_root, config);
    for result in rlibc_x2_results {
        let failed = !result.passed;
        results.push(result);
        if failed && config.fail_fast {
            return results;
        }
    }

    results
}

fn run_filtered_tests(config: &Config, workspace_root: &Path) -> Vec<TestResult> {
    let mut results = Vec::new();

    // Separate crates by target (musl vs default)
    let (musl_crates, default_crates): (Vec<_>, Vec<_>) = config
        .crates
        .iter()
        .partition(|c| c.contains("musl"));

    // Check if rlibc-x2 is requested (triggers rlibc-x2-tests)
    let run_rlibc_x2_tests_flag = config.crates.iter().any(|c| c == "rlibc-x2");

    // Run default target crates (each individually for clear per-crate results)
    for crate_name in &default_crates {
        print_section(&format!("cargo test ({crate_name})"));

        let args = vec!["-p", crate_name.as_str()];
        let result = run_cargo_test(&format!("cargo test ({crate_name})"), &args, workspace_root, config);
        let failed = !result.passed;
        results.push(result);
        if failed && config.fail_fast {
            return results;
        }
    }

    // Run musl target crates (each individually for clear per-crate results)
    for crate_name in &musl_crates {
        print_section(&format!("cargo test musl ({crate_name})"));

        let args = vec!["--target", "x86_64-unknown-linux-musl", "-p", crate_name.as_str()];
        let result = run_cargo_test(&format!("cargo test musl ({crate_name})"), &args, workspace_root, config);
        let failed = !result.passed;
        results.push(result);
        if failed && config.fail_fast {
            return results;
        }
    }

    // Run rlibc-x2-tests if rlibc-x2 was specified
    if run_rlibc_x2_tests_flag {
        print_section("rlibc-x2-tests");
        let rlibc_x2_results = run_rlibc_x2_tests(workspace_root, config);
        for result in rlibc_x2_results {
            let failed = !result.passed;
            results.push(result);
            if failed && config.fail_fast {
                return results;
            }
        }
    }

    results
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut args_iter = args.iter();

    // Parse subcommand (first argument)
    let subcommand = match args_iter.next().map(|s| s.as_str()) {
        Some("test") => Subcommand::Test,
        Some("-h" | "--help") => {
            print_main_help();
            std::process::exit(0);
        }
        Some(other) => {
            eprintln!("Unknown command: {other}");
            eprintln!("Use --help for usage information");
            std::process::exit(1);
        }
        None => {
            print_main_help();
            std::process::exit(0);
        }
    };

    let mut config = Config {
        subcommand,
        verbose: false,
        fail_fast: false,
        debug: false,
        crates: Vec::new(),
    };

    // Parse options and positional arguments for the subcommand
    for arg in args_iter {
        match arg.as_str() {
            "-v" | "--verbose" => config.verbose = true,
            "-f" | "--fail-fast" | "--fail" => config.fail_fast = true,
            "-r" | "--release" => config.debug = false,
            "-d" | "--debug" => config.debug = true,
            "-h" | "--help" => {
                print_test_help();
                std::process::exit(0);
            }
            other if other.starts_with('-') => {
                eprintln!("Unknown option: {other}");
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
            "." => {
                match resolve_current_crate() {
                    Ok(Some(name)) => config.crates.push(name),
                    Ok(None) => {} // Workspace root - leave crates empty for "all"
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                }
            }
            crate_name => {
                config.crates.push(crate_name.to_string());
            }
        }
    }

    config
}

/// Resolve "." to the crate name in the current directory
/// Returns None if at workspace root (meaning "all"), Some(name) for a crate, or error
fn resolve_current_crate() -> Result<Option<String>, String> {
    let cargo_toml = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?
        .join("Cargo.toml");

    if !cargo_toml.exists() {
        return Err("No Cargo.toml found in current directory".to_string());
    }

    let content = fs::read_to_string(&cargo_toml)
        .map_err(|e| format!("Failed to read Cargo.toml: {e}"))?;

    // Simple parsing - look for name = "..." in [package] section
    let mut in_package = false;
    for line in content.lines() {
        let line = line.trim();
        if line == "[package]" {
            in_package = true;
            continue;
        }
        if line.starts_with('[') {
            in_package = false;
            continue;
        }
        if in_package && line.starts_with("name") {
            // Parse: name = "crate-name"
            if let Some(eq_pos) = line.find('=') {
                let value = line[eq_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                return Ok(Some(value.to_string()));
            }
        }
    }

    // No [package] section - likely workspace root, treat as "all"
    Ok(None)
}

fn print_main_help() {
    println!("Usage: cargo xtask <COMMAND> [OPTIONS]");
    println!();
    println!("Commands:");
    println!("  test    Run all repository tests");
    println!();
    println!("Options:");
    println!("  -h, --help    Show this help");
    println!();
    println!("Run 'cargo xtask <COMMAND> --help' for command-specific options");
}

fn print_test_help() {
    println!("Usage: cargo xtask test [OPTIONS] [CRATES]...");
    println!();
    println!("Run repository tests. With no crates specified, runs all tests.");
    println!("With crates specified, tests only those crates.");
    println!();
    println!("Arguments:");
    println!("  [CRATES]...       Crates to test (e.g., ex-x1, hw-x2, rlibc-x2)");
    println!("                    Use '.' for the crate in the current directory");
    println!("                    Musl target auto-detected from crate name");
    println!("                    Specifying rlibc-x2 also runs rlibc-x2-tests");
    println!();
    println!("Options:");
    println!("  -v, --verbose     Show full test output");
    println!("  -f, --fail-fast   Stop on first failure");
    println!("  -r, --release     Use release builds (default)");
    println!("  -d, --debug       Use debug builds (faster iteration)");
    println!("  -h, --help        Show this help");
    println!();
    println!("Examples:");
    println!("  cargo xtask test              # all tests");
    println!("  cargo xtask test .            # test crate in current directory");
    println!("  cargo xtask test ex-x1        # test ex-x1 only");
    println!("  cargo xtask test ex-musl      # test ex-musl (musl target)");
    println!("  cargo xtask test rlibc-x2     # test rlibc-x2 + rlibc-x2-tests");
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
