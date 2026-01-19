//! Disassembly and reference counting

use crate::types::FunctionInfo;
use anyhow::Result;
use capstone::prelude::*;
use goblin::elf::Elf;
use std::collections::HashMap;

/// Count call references to functions using disassembly
pub fn count_references(
    verbose: bool,
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

        let insns = cs
            .disasm_all(section_data, section_addr)
            .map_err(|e| anyhow::anyhow!("Disassembly failed: {}", e))?;

        for insn in insns.iter() {
            let mnemonic = insn.mnemonic().unwrap_or("");

            // Look for call instructions
            if mnemonic == "call" || mnemonic == "callq" {
                if let Some(target) = extract_call_target(&insn) {
                    if let Some(func) = functions.get_mut(&target) {
                        func.references += 1;
                        if verbose {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::build_test_binary;

    #[test]
    fn test_count_references_finds_calls() {
        let binary_path = build_test_binary();
        let binary_data = std::fs::read(&binary_path).expect("Failed to read binary");
        let elf = Elf::parse(&binary_data).expect("Failed to parse ELF");

        // Collect some functions
        let mut functions: HashMap<u64, FunctionInfo> = HashMap::new();
        for sym in &elf.syms {
            if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
                let name = elf.strtab.get_at(sym.st_name).unwrap_or("???");
                functions.insert(
                    sym.st_value,
                    FunctionInfo {
                        name: name.to_string(),
                        size: sym.st_size,
                        address: sym.st_value,
                        references: 0,
                        source: None,
                    },
                );
            }
        }

        // Also check dynsym
        for sym in &elf.dynsyms {
            if sym.st_type() == goblin::elf::sym::STT_FUNC && sym.st_size > 0 {
                let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or("???");
                functions.entry(sym.st_value).or_insert(FunctionInfo {
                    name: name.to_string(),
                    size: sym.st_size,
                    address: sym.st_value,
                    references: 0,
                    source: None,
                });
            }
        }

        let initial_count = functions.len();
        assert!(initial_count > 0, "Should have found some functions");

        // Count references
        count_references(false, &elf, &binary_data, &mut functions).unwrap();

        // At least some functions should have references
        let total_refs: usize = functions.values().map(|f| f.references).sum();
        assert!(total_refs > 0, "Should have found some call references");
    }
}
