//! Project automation tasks
//!
//! All commands default to debug builds and verbose output.
//! Use `-r` for release builds, `-q` for quiet output.
//!
//! # Usage
//!
//! ```text
//! cargo xtask build                 # build all crates
//! cargo xtask build ex-x1 -r        # build specific crate (release)
//!
//! cargo xtask run hw-x1             # run specific crate (shows exit code)
//! cargo xtask run .                 # run crate in current directory
//!
//! cargo xtask test                  # test all crates
//! cargo xtask test ex-x1 -q         # test specific crate (quiet)
//! cargo xtask test rlibc-x2         # includes rlibc-x2-tests binaries
//! ```
//!
//! # Available Commands
//!
//! - `build` - Build crates (one at a time)
//! - `run` - Build and run crates (shows exit code)
//! - `test` - Run tests (with summary)

use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

enum Subcommand {
    Build,
    Run,
    Test,
}

struct Config {
    subcommand: Subcommand,
    quiet: bool,
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

    let cmd_name = match config.subcommand {
        Subcommand::Build => "build",
        Subcommand::Run => "run",
        Subcommand::Test => "test",
    };

    // Get list of crates to operate on
    let crates = if config.crates.is_empty() {
        get_all_crates(workspace_root)
    } else {
        config.crates.clone()
    };

    // Run command on each crate
    let mut results = Vec::new();
    for crate_name in &crates {
        // Skip lib-only crates for run command
        if cmd_name == "run" && !crate_has_binary(crate_name, workspace_root) {
            continue;
        }

        let result = run_cargo_on_crate(cmd_name, crate_name, workspace_root, &config);
        let failed = !result.passed;
        results.push(result);
        if failed && config.fail_fast {
            break;
        }
    }

    // For test, also run rlibc-x2-tests if rlibc-x2 was included
    if matches!(config.subcommand, Subcommand::Test) {
        let should_run_rlibc_x2_tests = if config.crates.is_empty() {
            true // Running all tests
        } else {
            config.crates.iter().any(|c| c == "rlibc-x2")
        };

        if should_run_rlibc_x2_tests {
            let rlibc_x2_results = run_rlibc_x2_tests(workspace_root, &config);
            for result in rlibc_x2_results {
                let failed = !result.passed;
                results.push(result);
                if failed && config.fail_fast {
                    break;
                }
            }
        }
    }

    // Only show summary for test command
    if matches!(config.subcommand, Subcommand::Test) {
        print_summary(&results)
    } else {
        // For build/run, just return success/failure based on results
        if results.iter().any(|r| !r.passed) {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        }
    }
}

/// Get list of all crates in the workspace (excludes xtask and test crates)
fn get_all_crates(workspace_root: &Path) -> Vec<String> {
    // These are the crates we want to build/run/test
    let crates = [
        "ex-x1",
        "ex-x2",
        "ex-glibc",
        "ex-musl",
        "hw-x1",
        "hw-x2",
        "hw-glibc",
        "hw-musl",
        "rlibc-x1",
        "rlibc-x2",
        "is-libc-used",
    ];

    // Filter to crates that exist
    crates
        .iter()
        .filter(|name| {
            get_crate_path(name, workspace_root)
                .join("Cargo.toml")
                .exists()
        })
        .map(|s| s.to_string())
        .collect()
}

/// Get the path to a crate directory
fn get_crate_path(name: &str, workspace_root: &Path) -> std::path::PathBuf {
    if name.starts_with("ex-") || name.starts_with("hw-") {
        workspace_root.join("apps").join(name)
    } else if name == "is-libc-used" {
        workspace_root.join("tools").join(name)
    } else {
        workspace_root.join(name)
    }
}

/// Check if a crate has a binary (src/main.rs)
fn crate_has_binary(name: &str, workspace_root: &Path) -> bool {
    get_crate_path(name, workspace_root)
        .join("src/main.rs")
        .exists()
}

/// Run a cargo command on a single crate
fn run_cargo_on_crate(
    cmd_name: &str,
    crate_name: &str,
    workspace_root: &Path,
    config: &Config,
) -> TestResult {
    let is_musl = crate_name.contains("musl");
    let target_info = if is_musl { " (musl)" } else { "" };

    println!("=== {crate_name}{target_info} ===");

    let mut cmd = Command::new("cargo");
    cmd.arg(cmd_name);

    if !config.debug {
        cmd.arg("--release");
    }

    cmd.args(["-p", crate_name]);

    if is_musl {
        cmd.args(["--target", "x86_64-unknown-linux-musl"]);
    }

    cmd.current_dir(workspace_root);

    if config.quiet {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        match cmd.output() {
            Ok(output) => {
                let passed = output.status.success();
                let exit_code = output.status.code().unwrap_or(-1);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if cmd_name == "run" {
                    println!("  exit code: {exit_code}");
                } else if passed {
                    println!("  ok");
                } else {
                    println!("  FAILED");
                    eprintln!("{stderr}");
                }

                TestResult {
                    name: crate_name.to_string(),
                    passed,
                    output: stderr.to_string(),
                }
            }
            Err(e) => {
                eprintln!("  Failed to run cargo {cmd_name}: {e}");
                TestResult {
                    name: crate_name.to_string(),
                    passed: false,
                    output: e.to_string(),
                }
            }
        }
    } else {
        // Verbose mode - show all output
        let status = cmd.status();
        let (passed, exit_code) = match &status {
            Ok(s) => (s.success(), s.code().unwrap_or(-1)),
            Err(_) => (false, -1),
        };

        if cmd_name == "run" {
            println!("  exit code: {exit_code}");
        }
        println!();

        TestResult {
            name: crate_name.to_string(),
            passed,
            output: String::new(),
        }
    }
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut args_iter = args.iter();

    // Parse subcommand (first argument)
    let subcommand = match args_iter.next().map(|s| s.as_str()) {
        Some("build") => Subcommand::Build,
        Some("run") => Subcommand::Run,
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
        quiet: false,
        fail_fast: false,
        debug: true, // Default to debug for faster iteration; use -r for release
        crates: Vec::new(),
    };

    // Parse options and positional arguments for the subcommand
    for arg in args_iter {
        match arg.as_str() {
            "-q" | "--quiet" => config.quiet = true,
            "-f" | "--fail-fast" | "--fail" => config.fail_fast = true,
            "-r" | "--release" => config.debug = false,
            "-d" | "--debug" => config.debug = true,
            "-h" | "--help" => {
                match config.subcommand {
                    Subcommand::Build => print_cmd_help("build"),
                    Subcommand::Run => print_cmd_help("run"),
                    Subcommand::Test => print_test_help(),
                }
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

    let content =
        fs::read_to_string(&cargo_toml).map_err(|e| format!("Failed to read Cargo.toml: {e}"))?;

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
    println!("  build   Build crates");
    println!("  run     Build and run crates");
    println!("  test    Run tests");
    println!();
    println!("Options:");
    println!("  -h, --help    Show this help");
    println!();
    println!("Run 'cargo xtask <COMMAND> --help' for command-specific options");
}

fn print_cmd_help(cmd: &str) {
    println!("Usage: cargo xtask {cmd} [OPTIONS] [CRATES]...");
    println!();
    println!("Run 'cargo {cmd}' on crates. With no crates specified, {cmd}s all.");
    println!();
    println!("Arguments:");
    println!("  [CRATES]...       Crates to {cmd} (e.g., ex-x1, hw-x2)");
    println!("                    Use '.' for the crate in the current directory");
    println!("                    Musl target auto-detected from crate name");
    println!();
    println!("Options:");
    println!("  -q, --quiet       Suppress output (default is verbose)");
    println!("  -f, --fail-fast   Stop on first failure");
    println!("  -d, --debug       Use debug builds (default)");
    println!("  -r, --release     Use release builds");
    println!("  -h, --help        Show this help");
    println!();
    println!("Examples:");
    println!("  cargo xtask {cmd}             # {cmd} all");
    println!("  cargo xtask {cmd} .           # {cmd} crate in current directory");
    println!("  cargo xtask {cmd} ex-x1       # {cmd} ex-x1 only");
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
    println!("  -q, --quiet       Suppress output (default is verbose)");
    println!("  -f, --fail-fast   Stop on first failure");
    println!("  -d, --debug       Use debug builds (default)");
    println!("  -r, --release     Use release builds");
    println!("  -h, --help        Show this help");
    println!();
    println!("Examples:");
    println!("  cargo xtask test              # all tests");
    println!("  cargo xtask test .            # test crate in current directory");
    println!("  cargo xtask test ex-x1        # test ex-x1 only");
    println!("  cargo xtask test ex-musl      # test ex-musl (musl target)");
    println!("  cargo xtask test rlibc-x2     # test rlibc-x2 + rlibc-x2-tests");
}

fn run_rlibc_x2_tests(workspace_root: &Path, config: &Config) -> Vec<TestResult> {
    let mut results = Vec::new();

    println!("=== rlibc-x2-tests ===");

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
                    if !config.quiet && !stdout.is_empty() {
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
