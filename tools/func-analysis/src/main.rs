//! func-analysis: Analyze libc function sizes and reference counts in ELF binaries
//!
//! For statically linked binaries (rlibc-x): extracts function sizes and counts
//! call references to each function.
//!
//! For dynamically linked binaries (glibc): identifies imported libc functions
//! and counts references through PLT stubs.

use anyhow::{Context, Result};
use capstone::prelude::*;
use clap::{Parser, ValueEnum};
use goblin::elf::Elf;
use is_libc_used::is_libc_used_from_bytes;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "func-analysis")]
#[command(about = "Analyze libc function sizes and reference counts in ELF binaries")]
struct Args {
    /// Path to the ELF binary to analyze
    binary: PathBuf,

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

    /// Include all symbols (not just libc-related)
    #[arg(short, long)]
    all: bool,

    /// Verbose output (show disassembly details)
    #[arg(short, long)]
    verbose: bool,

    /// Path to glibc library for size lookup (for dynamic binaries)
    #[arg(long)]
    libc_path: Option<PathBuf>,
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

#[derive(Debug, Clone, Serialize)]
struct FunctionInfo {
    name: String,
    size: u64,
    address: u64,
    references: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>, // "local" or "glibc" etc.
}

#[derive(Debug, Serialize)]
struct AnalysisResult {
    binary_path: String,
    is_dynamic: bool,
    total_functions: usize,
    total_code_size: u64,
    functions: Vec<FunctionInfo>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let binary_data = fs::read(&args.binary)
        .with_context(|| format!("Failed to read binary: {:?}", args.binary))?;

    let elf = Elf::parse(&binary_data)
        .with_context(|| "Failed to parse ELF binary")?;

    let is_dynamic = is_libc_used_from_bytes(&binary_data)
        .map(|r| r.uses_libc)
        .unwrap_or(false);

    let result = if is_dynamic {
        analyze_dynamic(&args, &elf, &binary_data)?
    } else {
        analyze_static(&args, &elf, &binary_data)?
    };

    output_result(&args, &result)?;

    Ok(())
}

/// Analyze a statically linked binary
fn analyze_static(args: &Args, elf: &Elf, binary_data: &[u8]) -> Result<AnalysisResult> {
    let mut functions: HashMap<u64, FunctionInfo> = HashMap::new();

    // Collect all function symbols
    for sym in &elf.syms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
            let name = elf.strtab.get_at(sym.st_name).unwrap_or("???");

            // Skip if filtering and doesn't match
            if let Some(ref filter) = args.filter {
                if !name.contains(filter) {
                    continue;
                }
            }

            // Skip internal rust/compiler symbols unless --all
            if !args.all && is_internal_symbol(name) {
                continue;
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
    count_references(args, elf, binary_data, &mut functions)?;

    // Convert to sorted vec
    let mut func_vec: Vec<_> = functions.into_values().collect();
    filter_and_sort(&args, &mut func_vec);

    let total_code_size = func_vec.iter().map(|f| f.size).sum();

    Ok(AnalysisResult {
        binary_path: args.binary.display().to_string(),
        is_dynamic: false,
        total_functions: func_vec.len(),
        total_code_size,
        functions: func_vec,
    })
}

/// Analyze a dynamically linked binary
fn analyze_dynamic(args: &Args, elf: &Elf, binary_data: &[u8]) -> Result<AnalysisResult> {
    let mut functions: HashMap<u64, FunctionInfo> = HashMap::new();

    // For dynamic binaries, we care about:
    // 1. Imported symbols (from .dynsym with UND section)
    // 2. PLT entries that reference them

    // Build a map of PLT entries to function names
    let plt_map = build_plt_map(elf)?;

    // Get sizes from libc if path provided
    let libc_sizes = if let Some(ref libc_path) = args.libc_path {
        load_libc_sizes(libc_path)?
    } else {
        HashMap::new()
    };

    // Collect imported libc functions
    for sym in &elf.dynsyms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC {
            let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");

            // Check if it's a GLIBC symbol
            let is_glibc = elf.dynstrtab.to_vec()?.iter()
                .any(|s| s.contains("GLIBC"));

            if !args.all && !is_glibc && sym.st_shndx != goblin::elf::section_header::SHN_UNDEF as usize {
                continue;
            }

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
    count_references(args, elf, binary_data, &mut functions)?;

    let mut func_vec: Vec<_> = functions.into_values().collect();
    filter_and_sort(&args, &mut func_vec);

    let total_code_size = func_vec.iter().map(|f| f.size).sum();

    Ok(AnalysisResult {
        binary_path: args.binary.display().to_string(),
        is_dynamic: true,
        total_functions: func_vec.len(),
        total_code_size,
        functions: func_vec,
    })
}

/// Build a map of function names to their PLT addresses
fn build_plt_map(elf: &Elf) -> Result<HashMap<String, u64>> {
    let mut plt_map = HashMap::new();

    // Find .rela.plt section for PLT relocations
    for reloc in &elf.pltrelocs {
        let sym_idx = reloc.r_sym;
        if let Some(sym) = elf.dynsyms.get(sym_idx) {
            let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");

            // PLT entry address: typically the relocation target + PLT base
            // This is simplified; real PLT layout varies
            if let Some(plt_shdr) = elf.section_headers.iter()
                .find(|sh| {
                    elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("") == ".plt"
                })
            {
                // Each PLT entry is typically 16 bytes on x86_64
                // First entry is resolver, real entries start at +16
                let plt_entry_addr = plt_shdr.sh_addr + 16 + (plt_map.len() as u64 * 16);
                plt_map.insert(name.to_string(), plt_entry_addr);
            }
        }
    }

    Ok(plt_map)
}

/// Load function sizes from a glibc shared library
fn load_libc_sizes(libc_path: &PathBuf) -> Result<HashMap<String, u64>> {
    let data = fs::read(libc_path)
        .with_context(|| format!("Failed to read libc: {:?}", libc_path))?;

    let elf = Elf::parse(&data)?;
    let mut sizes = HashMap::new();

    for sym in &elf.dynsyms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
            let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");
            sizes.insert(name.to_string(), sym.st_size);
        }
    }

    // Also check regular symtab if available
    for sym in &elf.syms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
            let name = elf.strtab.get_at(sym.st_name).unwrap_or("???");
            sizes.entry(name.to_string()).or_insert(sym.st_size);
        }
    }

    Ok(sizes)
}

/// Count call references to functions using disassembly
fn count_references(
    args: &Args,
    elf: &Elf,
    binary_data: &[u8],
    functions: &mut HashMap<u64, FunctionInfo>,
) -> Result<()> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create Capstone: {}", e))?;

    // Find executable sections
    for shdr in &elf.section_headers {
        // Check if section is executable (SHF_EXECINSTR = 0x4)
        if shdr.sh_flags & 0x4 == 0 {
            continue;
        }

        let section_name = elf.shdr_strtab.get_at(shdr.sh_name).unwrap_or("");

        // Skip PLT itself to avoid double-counting
        if section_name == ".plt" || section_name == ".plt.got" {
            continue;
        }

        let start = shdr.sh_offset as usize;
        let end = start + shdr.sh_size as usize;

        if end > binary_data.len() {
            continue;
        }

        let section_data = &binary_data[start..end];
        let section_addr = shdr.sh_addr;

        let insns = cs.disasm_all(section_data, section_addr)
            .map_err(|e| anyhow::anyhow!("Disassembly failed: {}", e))?;

        for insn in insns.iter() {
            let mnemonic = insn.mnemonic().unwrap_or("");

            // Look for call and jmp instructions
            if mnemonic == "call" || mnemonic == "callq" {
                if let Some(target) = extract_call_target(&insn) {
                    if let Some(func) = functions.get_mut(&target) {
                        func.references += 1;
                        if args.verbose {
                            eprintln!(
                                "  0x{:x}: {} -> {} (0x{:x})",
                                insn.address(),
                                mnemonic,
                                func.name,
                                target
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Extract the target address from a call instruction
fn extract_call_target(insn: &capstone::Insn) -> Option<u64> {
    let op_str = insn.op_str()?;

    // Direct call: "0x1234" or "symbol"
    // We need the absolute address

    // For relative calls, capstone gives us the computed target
    // Check if it's a hex address
    if op_str.starts_with("0x") || op_str.starts_with("0X") {
        u64::from_str_radix(&op_str[2..], 16).ok()
    } else if let Ok(addr) = op_str.parse::<u64>() {
        Some(addr)
    } else {
        // For PC-relative calls, we need to compute the target
        // Format might be like "0x401234" after capstone processes it
        // or it might be an indirect call like "*%rax" which we skip
        if op_str.starts_with('*') {
            None // indirect call
        } else {
            // Try parsing as hex without prefix
            u64::from_str_radix(op_str.trim(), 16).ok()
        }
    }
}

/// Check if a symbol is internal (compiler/runtime generated)
fn is_internal_symbol(name: &str) -> bool {
    // Skip internal symbols unless user wants all
    name.starts_with("_ZN") || // Rust mangled
    name.starts_with("__rust") ||
    name.starts_with(".L") ||
    name.starts_with("GCC_except") ||
    name.starts_with("__gcc") ||
    name.starts_with("_GLOBAL") ||
    name.contains("cgu") || // codegen unit
    name == "_start" ||
    name == "__libc_csu_init" ||
    name == "__libc_csu_fini"
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
            println!("Binary: {}", result.binary_path);
            println!("Type: {}", if result.is_dynamic { "dynamic" } else { "static" });
            println!("Functions: {}", result.total_functions);
            println!("Total code size: {} bytes", result.total_code_size);
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
