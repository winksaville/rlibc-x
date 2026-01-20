//! Binary analysis functions for func-analysis

use crate::types::{AnalysisResult, FunctionInfo};
use crate::{disasm, elf_utils, output};
use anyhow::Result;
use goblin::elf::Elf;
use output::SortBy;
use std::collections::HashMap;
use std::path::Path;

/// Analyze a statically linked binary
pub fn analyze_static(
    binary_path: &Path,
    elf: &Elf,
    binary_data: &[u8],
    filter: Option<&str>,
    verbose: bool,
    min_refs: usize,
    sort: SortBy,
) -> Result<AnalysisResult> {
    let mut functions: HashMap<u64, FunctionInfo> = HashMap::new();

    // Collect function symbols from .symtab
    for sym in &elf.syms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
            let name = elf.strtab.get_at(sym.st_name).unwrap_or("???");

            // Skip if filtering and doesn't match
            if let Some(f) = filter {
                if !name.contains(f) {
                    continue;
                }
            }

            functions.insert(
                sym.st_value,
                FunctionInfo {
                    name: name.to_string(),
                    size: sym.st_size,
                    address: sym.st_value,
                    references: 0,
                    source: Some("local".to_string()),
                },
            );
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
            if let Some(f) = filter {
                if !name.contains(f) {
                    continue;
                }
            }

            functions.insert(
                sym.st_value,
                FunctionInfo {
                    name: name.to_string(),
                    size: sym.st_size,
                    address: sym.st_value,
                    references: 0,
                    source: Some("local".to_string()),
                },
            );
        }
    }

    // Count references by disassembling
    disasm::count_references(verbose, elf, binary_data, &mut functions)?;

    // Convert to sorted vec
    let mut func_vec: Vec<_> = functions.into_values().collect();
    output::filter_and_sort(min_refs, sort, &mut func_vec);

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
pub fn analyze_dynamic(
    binary_path: &Path,
    elf: &Elf,
    binary_data: &[u8],
    filter: Option<&str>,
    verbose: bool,
    min_refs: usize,
    sort: SortBy,
    libc_path: Option<&Path>,
) -> Result<AnalysisResult> {
    let mut functions: HashMap<u64, FunctionInfo> = HashMap::new();

    // For dynamic binaries, we care about:
    // 1. Local functions defined in the binary (from .symtab)
    // 2. Imported symbols (from .dynsym with UND section)
    // 3. PLT/GOT entries that reference imported functions

    // First, collect LOCAL functions from .symtab (app's own functions)
    for sym in &elf.syms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
            let name = elf.strtab.get_at(sym.st_name).unwrap_or("???");

            // Skip if filtering and doesn't match
            if let Some(f) = filter {
                if !name.contains(f) {
                    continue;
                }
            }

            functions.insert(
                sym.st_value,
                FunctionInfo {
                    name: name.to_string(),
                    size: sym.st_size,
                    address: sym.st_value,
                    references: 0,
                    source: Some("local".to_string()),
                },
            );
        }
    }

    // Also check .dynsym for defined (non-UND) functions with size
    for sym in &elf.dynsyms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC
            && sym.st_size > 0
            && sym.st_shndx != goblin::elf::section_header::SHN_UNDEF as usize
        {
            // Skip if we already have this address from symtab
            if functions.contains_key(&sym.st_value) {
                continue;
            }

            let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");

            // Skip if filtering and doesn't match
            if let Some(f) = filter {
                if !name.contains(f) {
                    continue;
                }
            }

            functions.insert(
                sym.st_value,
                FunctionInfo {
                    name: name.to_string(),
                    size: sym.st_size,
                    address: sym.st_value,
                    references: 0,
                    source: Some("local".to_string()),
                },
            );
        }
    }

    // Build a map of PLT/GOT entries to function names for imported functions
    let plt_map = elf_utils::build_plt_map(elf)?;

    // Get sizes from libc - use provided path or auto-detect
    let libc_sizes = if let Some(path) = libc_path {
        elf_utils::load_libc_sizes(path)?
    } else if let Some(detected) = elf_utils::detect_system_libc() {
        if verbose {
            eprintln!("Auto-detected libc: {:?}", detected);
        }
        elf_utils::load_libc_sizes(&detected).unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Collect IMPORTED libc functions (UND symbols)
    for sym in &elf.dynsyms {
        if sym.st_type() == goblin::elf::sym::STT_FUNC {
            let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");

            if sym.st_shndx == goblin::elf::section_header::SHN_UNDEF as usize {
                // Skip if filtering and doesn't match
                if let Some(f) = filter {
                    if !name.contains(f) {
                        continue;
                    }
                }

                // Find PLT/GOT address for this symbol
                let plt_addr = plt_map.get(name).copied();

                let size = libc_sizes.get(name).copied().unwrap_or(0);

                if let Some(addr) = plt_addr {
                    functions.insert(
                        addr,
                        FunctionInfo {
                            name: name.to_string(),
                            size,
                            address: addr,
                            references: 0,
                            source: Some("glibc".to_string()),
                        },
                    );
                }
            }
        }
    }

    // Count references to PLT stubs
    disasm::count_references(verbose, elf, binary_data, &mut functions)?;

    let mut func_vec: Vec<_> = functions.into_values().collect();
    output::filter_and_sort(min_refs, sort, &mut func_vec);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::build_test_binary;
    use std::fs;

    #[test]
    fn test_analyze_static_finds_functions() {
        let binary_path = build_test_binary();
        let binary_data = fs::read(&binary_path).expect("Failed to read binary");
        let elf = Elf::parse(&binary_data).expect("Failed to parse ELF");

        let result = analyze_static(
            &binary_path,
            &elf,
            &binary_data,
            None,
            false,
            0,
            SortBy::Refs,
        )
        .expect("Analysis should succeed");

        assert!(!result.is_dynamic);
        assert!(result.total_functions > 0, "Should find functions");
        assert!(result.text_section_size > 0, "Should have .text section");
    }

    #[test]
    fn test_analyze_static_with_filter() {
        let binary_path = build_test_binary();
        let binary_data = fs::read(&binary_path).expect("Failed to read binary");
        let elf = Elf::parse(&binary_data).expect("Failed to parse ELF");

        let result = analyze_static(
            &binary_path,
            &elf,
            &binary_data,
            Some("main"),
            false,
            0,
            SortBy::Refs,
        )
        .expect("Analysis should succeed");

        // Should only have functions containing "main"
        for func in &result.functions {
            assert!(
                func.name.contains("main"),
                "Function {} should contain 'main'",
                func.name
            );
        }
    }
}
