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

/// Build a map of function names to their PLT addresses
pub fn build_plt_map(elf: &Elf) -> Result<HashMap<String, u64>> {
    let mut plt_map = HashMap::new();

    // Find .rela.plt section for PLT relocations
    for reloc in &elf.pltrelocs {
        let sym_idx = reloc.r_sym;
        if let Some(sym) = elf.dynsyms.get(sym_idx) {
            let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");

            // PLT entry address: typically the relocation target + PLT base
            // This is simplified; real PLT layout varies
            if let Some(plt_shdr) = elf
                .section_headers
                .iter()
                .find(|sh| elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("") == ".plt")
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
