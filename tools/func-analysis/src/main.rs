//! func-analysis: Analyze libc function sizes and reference counts in ELF binaries
//!
//! For statically linked binaries (rlibc-x): extracts function sizes and counts
//! call references to each function.
//!
//! For dynamically linked binaries (glibc): identifies imported libc functions
//! and counts references through PLT stubs.

mod disasm;
mod elf_utils;
#[cfg(test)]
mod test_utils;
mod types;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use goblin::elf::Elf;
use is_libc_used::is_libc_used_from_bytes;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SortBy {
    Name,
    Size,
    Refs,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle subcommands
    let binary: PathBuf = match &args.command {
        Command::Compare { funcs_file, binary1, binary2 } => {
            return run_compare(funcs_file, binary1, binary2);
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

    output_result(&args, &result)?;

    Ok(())
}

/// Run compare mode: compare function sizes between two binaries
fn run_compare(funcs_file: &PathBuf, binary1: &PathBuf, binary2: &PathBuf) -> Result<()> {
    // Read function names from file
    let file = fs::File::open(funcs_file)
        .with_context(|| format!("Failed to open functions file: {:?}", funcs_file))?;
    let reader = BufReader::new(file);
    let func_names: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect();

    // Get function sizes from both binaries
    let sizes1 = get_function_sizes(binary1)?;
    let sizes2 = get_function_sizes(binary2)?;

    // Get binary names for header
    let name1 = binary1.file_name().unwrap_or_default().to_string_lossy();
    let name2 = binary2.file_name().unwrap_or_default().to_string_lossy();

    // Print comparison table
    println!("{:<40} {:>12} {:>12} {:>10}", "FUNCTION", name1, name2, "RATIO");
    println!("{}", "-".repeat(76));

    let mut total1: u64 = 0;
    let mut total2: u64 = 0;
    let mut compared = 0;

    for func in &func_names {
        let size1 = sizes1.get(func).copied();
        let size2 = sizes2.get(func).copied();

        let s1_str = size1.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string());
        let s2_str = size2.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string());

        let ratio_str = match (size1, size2) {
            (Some(s1), Some(s2)) if s1 > 0 => {
                total1 += s1;
                total2 += s2;
                compared += 1;
                format!("{:.1}x", s2 as f64 / s1 as f64)
            }
            _ => "-".to_string(),
        };

        println!("{:<40} {:>12} {:>12} {:>10}", func, s1_str, s2_str, ratio_str);
    }

    println!("{}", "-".repeat(76));
    println!("{:<40} {:>12} {:>12} {:>10}",
        format!("TOTAL ({} functions)", compared),
        total1,
        total2,
        if total1 > 0 { format!("{:.1}x", total2 as f64 / total1 as f64) } else { "-".to_string() }
    );

    Ok(())
}

/// Extract function name -> size map from a binary
fn get_function_sizes(binary: &PathBuf) -> Result<HashMap<String, u64>> {
    let binary_data = fs::read(binary)
        .with_context(|| format!("Failed to read binary: {:?}", binary))?;

    let elf = Elf::parse(&binary_data)
        .with_context(|| format!("Failed to parse ELF: {:?}", binary))?;

    let mut sizes = HashMap::new();

    // Check .symtab
    for sym in &elf.syms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
            let name = elf.strtab.get_at(sym.st_name).unwrap_or("???");
            sizes.insert(name.to_string(), sym.st_size);
        }
    }

    // Check .dynsym
    for sym in &elf.dynsyms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
            let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");
            sizes.entry(name.to_string()).or_insert(sym.st_size);
        }
    }

    Ok(sizes)
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
    filter_and_sort(&args, &mut func_vec);

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
    filter_and_sort(&args, &mut func_vec);

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

fn filter_and_sort(args: &Args, functions: &mut Vec<FunctionInfo>) {
    // Filter by min refs
    functions.retain(|f| f.references >= args.min_refs);

    // Sort
    match args.sort {
        SortBy::Name => functions.sort_by(|a, b| a.name.cmp(&b.name)),
        SortBy::Size => functions.sort_by(|a, b| b.size.cmp(&a.size)),
        SortBy::Refs => functions.sort_by(|a, b| b.references.cmp(&a.references)),
    }
}

fn output_result(args: &Args, result: &AnalysisResult) -> Result<()> {
    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        OutputFormat::Csv => {
            println!("name,size,address,references,source");
            for f in &result.functions {
                println!(
                    "{},{},{:#x},{},{}",
                    f.name,
                    f.size,
                    f.address,
                    f.references,
                    f.source.as_deref().unwrap_or("")
                );
            }
        }
        OutputFormat::Table => {
            let coverage = if result.text_section_size > 0 {
                (result.total_code_size as f64 / result.text_section_size as f64) * 100.0
            } else {
                0.0
            };

            println!("Binary: {}", result.binary_path);
            println!("Type: {}", if result.is_dynamic { "dynamic" } else { "static" });
            println!("Functions: {}", result.total_functions);
            println!(".text size: {} bytes", result.text_section_size);
            println!("Total code size: {} bytes ({:.1}% coverage)", result.total_code_size, coverage);
            println!();
            println!("{:<40} {:>10} {:>12} {:>8}", "FUNCTION", "SIZE", "ADDRESS", "REFS");
            println!("{}", "-".repeat(74));

            for f in &result.functions {
                println!(
                    "{:<40} {:>10} {:#12x} {:>8}",
                    truncate(&f.name, 40),
                    f.size,
                    f.address,
                    f.references
                );
            }
        }
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
