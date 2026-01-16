//! CLI tool to check if a binary uses libc.
//!
//! Exit codes:
//!   0 - Binary does NOT use libc
//!   1 - Binary DOES use libc
//!   2 - Error (file not found, parse error, etc.)

use is_libc_used::is_libc_used;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    let mut verbose = false;
    let mut path: Option<&str> = None;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => {
                print_usage(&args[0]);
                return ExitCode::from(0);
            }
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {}", s);
                print_usage(&args[0]);
                return ExitCode::from(2);
            }
            s => {
                if path.is_some() {
                    eprintln!("Error: Multiple paths provided");
                    print_usage(&args[0]);
                    return ExitCode::from(2);
                }
                path = Some(s);
            }
        }
    }

    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("Error: No binary path provided");
            print_usage(&args[0]);
            return ExitCode::from(2);
        }
    };

    let path = Path::new(path);

    if !path.exists() {
        eprintln!("Error: File not found: {}", path.display());
        return ExitCode::from(2);
    }

    match is_libc_used(path) {
        Ok(result) => {
            if verbose {
                for line in &result.info {
                    eprintln!("{}", line);
                }
            }

            if result.uses_libc {
                if verbose {
                    eprintln!("Result: uses libc");
                }
                ExitCode::from(1)
            } else {
                if verbose {
                    eprintln!("Result: no libc");
                }
                ExitCode::from(0)
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(2)
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} [OPTIONS] <binary>", program);
    eprintln!();
    eprintln!("Check if an ELF binary uses libc.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -v, --verbose  Print detailed check information");
    eprintln!("  -h, --help     Print this help message");
    eprintln!();
    eprintln!("Exit codes:");
    eprintln!("  0  Binary does NOT use libc");
    eprintln!("  1  Binary DOES use libc");
    eprintln!("  2  Error (file not found, parse error, etc.)");
}
