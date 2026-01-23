//! func-analysis: Analyze libc function sizes and reference counts in ELF binaries
//!
//! For statically linked binaries (rlibc-x): extracts function sizes and counts
//! call references to each function.
//!
//! For dynamically linked binaries (glibc): identifies imported libc functions
//! and counts references through PLT stubs.

mod analyze;
mod compare;
mod disasm;
mod elf_utils;
mod output;
#[cfg(test)]
mod test_utils;
mod types;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use goblin::elf::Elf;
use is_libc_used::is_libc_used_from_bytes;
use output::{OutputFormat, SortBy};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "func-analysis")]
#[command(about = "Analyze libc function sizes and reference counts in ELF binaries")]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// Filter functions by name (substring match)
    #[arg(short = 'F', long)]
    filter: Option<String>,

    /// Show only functions with at least N references
    #[arg(long, default_value = "0")]
    min_refs: usize,

    /// Sort by: name, size, refs
    #[arg(short, long, default_value = "refs")]
    sort: SortBy,

    /// Verbose output (show disassembly details)
    #[arg(short, long)]
    verbose: bool,

    /// Path to glibc library for size lookup (for dynamic binaries)
    #[arg(long)]
    libc_path: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Analyze function sizes and references in a binary
    Analyze {
        /// Binary to analyze
        binary: PathBuf,
    },
    /// Compare function sizes between two binaries
    Compare {
        /// File containing function names (one per line)
        funcs_file: PathBuf,
        /// First binary to analyze
        binary1: PathBuf,
        /// Second binary to analyze
        binary2: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle subcommands
    let binary: PathBuf = match &args.command {
        Command::Compare {
            funcs_file,
            binary1,
            binary2,
        } => {
            return compare::run_compare(funcs_file, binary1, binary2);
        }
        Command::Analyze { binary } => binary.clone(),
    };

    // Analyze mode
    let binary = &binary;
    let binary_data =
        fs::read(binary).with_context(|| format!("Failed to read binary: {:?}", binary))?;

    let elf = Elf::parse(&binary_data).with_context(|| "Failed to parse ELF binary")?;

    let is_dynamic = is_libc_used_from_bytes(&binary_data)
        .map(|r| r.uses_libc)
        .unwrap_or(false);

    let result = if is_dynamic {
        analyze::analyze_dynamic(
            binary,
            &elf,
            &binary_data,
            args.filter.as_deref(),
            args.verbose,
            args.min_refs,
            args.sort,
            args.libc_path.as_deref(),
        )?
    } else {
        analyze::analyze_static(
            binary,
            &elf,
            &binary_data,
            args.filter.as_deref(),
            args.verbose,
            args.min_refs,
            args.sort,
        )?
    };

    output::output_result(args.format, &result)?;

    Ok(())
}
