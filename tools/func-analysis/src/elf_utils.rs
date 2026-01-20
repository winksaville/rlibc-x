//! ELF utility functions for func-analysis

use anyhow::{Context, Result};
use goblin::elf::Elf;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Get the size of the .text section
pub fn get_text_section_size(elf: &Elf) -> u64 {
    elf.section_headers
        .iter()
        .find(|sh| elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("") == ".text")
        .map(|sh| sh.sh_size)
        .unwrap_or(0)
}

/// Build a map of function names to their GOT/PLT addresses
///
/// Handles two linking styles:
/// 1. Traditional PLT: .rela.plt with JUMP_SLOT relocations -> .plt stubs
/// 2. PLT-less/GOT-direct: .rela.dyn with GLOB_DAT relocations -> .got entries
pub fn build_plt_map(elf: &Elf) -> Result<HashMap<String, u64>> {
    use goblin::elf::reloc::R_X86_64_GLOB_DAT;

    let mut plt_map = HashMap::new();

    // Method 1: Traditional PLT via .rela.plt
    if let Some(plt_shdr) = elf
        .section_headers
        .iter()
        .find(|sh| elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("") == ".plt")
    {
        for reloc in &elf.pltrelocs {
            let sym_idx = reloc.r_sym;
            if let Some(sym) = elf.dynsyms.get(sym_idx) {
                let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");
                // Each PLT entry is typically 16 bytes on x86_64
                // First entry is resolver, real entries start at +16
                let plt_entry_addr = plt_shdr.sh_addr + 16 + (plt_map.len() as u64 * 16);
                plt_map.insert(name.to_string(), plt_entry_addr);
            }
        }
    }

    // Method 2: PLT-less linking via .rela.dyn GLOB_DAT relocations
    // Modern linkers may use direct GOT calls instead of PLT stubs
    if plt_map.is_empty() {
        for reloc in &elf.dynrelas {
            // Only handle GLOB_DAT relocations (direct GOT entries for functions)
            if reloc.r_type == R_X86_64_GLOB_DAT {
                let sym_idx = reloc.r_sym;
                if let Some(sym) = elf.dynsyms.get(sym_idx) {
                    // Only include function symbols
                    if sym.st_type() == goblin::elf::sym::STT_FUNC {
                        let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");
                        // Use the GOT entry address (reloc.r_offset) as the target
                        plt_map.insert(name.to_string(), reloc.r_offset);
                    }
                }
            }
        }
    }

    Ok(plt_map)
}

/// Auto-detect the system libc path using ldd
pub fn detect_system_libc() -> Option<std::path::PathBuf> {
    use std::process::Command;

    // Use ldd on a known binary to find libc
    let output = Command::new("ldd").arg("/bin/sh").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        // Look for: libc.so.6 => /usr/lib/libc.so.6 (0x...)
        if line.contains("libc.so") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[1] == "=>" {
                let path = std::path::PathBuf::from(parts[2]);
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

/// Load function sizes from a glibc shared library
pub fn load_libc_sizes(libc_path: &Path) -> Result<HashMap<String, u64>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::build_test_binary;

    #[test]
    fn test_get_text_section_size() {
        let binary_path = build_test_binary();
        let binary_data = fs::read(&binary_path).expect("Failed to read binary");
        let elf = Elf::parse(&binary_data).expect("Failed to parse ELF");

        let text_size = get_text_section_size(&elf);
        assert!(text_size > 0, ".text section should have non-zero size");
    }

    #[test]
    fn test_build_plt_map_static_binary() {
        // Static binaries shouldn't have PLT entries
        let binary_path = build_test_binary();
        let binary_data = fs::read(&binary_path).expect("Failed to read binary");
        let elf = Elf::parse(&binary_data).expect("Failed to parse ELF");

        let plt_map = build_plt_map(&elf).expect("build_plt_map should succeed");
        // Static binary (ex-x2) should have empty or minimal PLT
        // This is more of a smoke test
        assert!(plt_map.len() < 100, "Static binary shouldn't have many PLT entries");
    }
}
