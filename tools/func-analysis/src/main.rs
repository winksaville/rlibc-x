//! func-analysis: Analyze libc function sizes and reference counts in ELF binaries
//!
//! For statically linked binaries (rlibc-x): extracts function sizes and counts
//! call references to each function.
//!
//! For dynamically linked binaries (glibc): identifies imported libc functions
//! and counts references through PLT stubs.

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
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use types::{AnalysisResult, FunctionInfo};

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
        Command::Compare { funcs_file, binary1, binary2 } => {
            return compare::run_compare(funcs_file, binary1, binary2);
        }
        Command::Analyze { binary } => binary.clone(),
    };

    // Analyze mode
    let binary = &binary;
    let binary_data = fs::read(binary)
        .with_context(|| format!("Failed to read binary: {:?}", binary))?;

    let elf = Elf::parse(&binary_data)
        .with_context(|| "Failed to parse ELF binary")?;

    let is_dynamic = is_libc_used_from_bytes(&binary_data)
        .map(|r| r.uses_libc)
        .unwrap_or(false);

    let result = if is_dynamic {
        analyze_dynamic(&args, binary, &elf, &binary_data)?
    } else {
        analyze_static(&args, binary, &elf, &binary_data)?
    };

    output::output_result(args.format, &result)?;

    Ok(())
}

/// Analyze a statically linked binary
fn analyze_static(args: &Args, binary_path: &PathBuf, elf: &Elf, binary_data: &[u8]) -> Result<AnalysisResult> {
    let mut functions: HashMap<u64, FunctionInfo> = HashMap::new();

    // Collect function symbols from .symtab
    for sym in &elf.syms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
            let name = elf.strtab.get_at(sym.st_name).unwrap_or("???");

            // Skip if filtering and doesn't match
            if let Some(ref filter) = args.filter {
                if !name.contains(filter) {
                    continue;
                }
            }

            functions.insert(sym.st_value, FunctionInfo {
                name: name.to_string(),
                size: sym.st_size,
                address: sym.st_value,
                references: 0,
                source: Some("local".to_string()),
            });
        }
    }

    // Also check .dynsym (release builds may strip .symtab but keep .dynsym)
    for sym in &elf.dynsyms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
            // Skip if we already have this address from symtab
            if functions.contains_key(&sym.st_value) {
                continue;
            }

            let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");

            // Skip if filtering and doesn't match
            if let Some(ref filter) = args.filter {
                if !name.contains(filter) {
                    continue;
                }
            }

            functions.insert(sym.st_value, FunctionInfo {
                name: name.to_string(),
                size: sym.st_size,
                address: sym.st_value,
                references: 0,
                source: Some("local".to_string()),
            });
        }
    }

    // Count references by disassembling
    disasm::count_references(args.verbose, elf, binary_data, &mut functions)?;

    // Convert to sorted vec
    let mut func_vec: Vec<_> = functions.into_values().collect();
    output::filter_and_sort(args.min_refs, args.sort, &mut func_vec);

    let total_code_size = func_vec.iter().map(|f| f.size).sum();
    let text_section_size = elf_utils::get_text_section_size(elf);

    Ok(AnalysisResult {
        binary_path: binary_path.display().to_string(),
        is_dynamic: false,
        total_functions: func_vec.len(),
        total_code_size,
        text_section_size,
        functions: func_vec,
    })
}

/// Analyze a dynamically linked binary
fn analyze_dynamic(args: &Args, binary_path: &PathBuf, elf: &Elf, binary_data: &[u8]) -> Result<AnalysisResult> {
    let mut functions: HashMap<u64, FunctionInfo> = HashMap::new();

    // For dynamic binaries, we care about:
    // 1. Imported symbols (from .dynsym with UND section)
    // 2. PLT entries that reference them

    // Build a map of PLT entries to function names
    let plt_map = elf_utils::build_plt_map(elf)?;

    // Get sizes from libc if path provided
    let libc_sizes = if let Some(ref libc_path) = args.libc_path {
        elf_utils::load_libc_sizes(libc_path)?
    } else {
        HashMap::new()
    };

    // Collect imported libc functions
    for sym in &elf.dynsyms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC {
            let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");

            if sym.st_shndx == goblin::elf::section_header::SHN_UNDEF as usize {
                // Skip if filtering and doesn't match
                if let Some(ref filter) = args.filter {
                    if !name.contains(filter) {
                        continue;
                    }
                }

                // Find PLT address for this symbol
                let plt_addr = plt_map.get(name).copied();

                let size = libc_sizes.get(name).copied().unwrap_or(0);

                if let Some(addr) = plt_addr {
                    functions.insert(addr, FunctionInfo {
                        name: name.to_string(),
                        size,
                        address: addr,
                        references: 0,
                        source: Some("glibc".to_string()),
                    });
                }
            }
        }
    }

    // Count references to PLT stubs
    disasm::count_references(args.verbose, elf, binary_data, &mut functions)?;

    let mut func_vec: Vec<_> = functions.into_values().collect();
    output::filter_and_sort(args.min_refs, args.sort, &mut func_vec);

    let total_code_size = func_vec.iter().map(|f| f.size).sum();
    let text_section_size = elf_utils::get_text_section_size(elf);

    Ok(AnalysisResult {
        binary_path: binary_path.display().to_string(),
        is_dynamic: true,
        total_functions: func_vec.len(),
        total_code_size,
        text_section_size,
        functions: func_vec,
    })
}

