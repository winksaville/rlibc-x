//! Disassembly and reference counting using iced-x86

use crate::types::FunctionInfo;
use anyhow::Result;
use goblin::elf::Elf;
use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction};
use std::collections::HashMap;

/// Count call references to functions using disassembly
pub fn count_references(
    verbose: bool,
    elf: &Elf,
    binary_data: &[u8],
    functions: &mut HashMap<u64, FunctionInfo>,
) -> Result<()> {
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

        // Create decoder for x86-64
        let mut decoder = Decoder::with_ip(64, section_data, section_addr, DecoderOptions::NONE);

        let mut instruction = Instruction::default();
        while decoder.can_decode() {
            decoder.decode_out(&mut instruction);

            // Check if this is a call instruction
            if instruction.flow_control() == FlowControl::Call {
                if let Some(target) = extract_call_target(&instruction) {
                    if let Some(func) = functions.get_mut(&target) {
                        func.references += 1;
                        if verbose {
                            eprintln!(
                                "  0x{:x}: call -> {} (0x{:x})",
                                instruction.ip(),
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
fn extract_call_target(instruction: &Instruction) -> Option<u64> {
    // For near relative calls (CALL rel32), get the computed target
    if instruction.is_call_near_indirect() {
        // Indirect calls like `call *%rax` - we can't resolve these statically
        None
    } else {
        // Direct calls - iced-x86 computes the absolute target for us
        let target = instruction.near_branch_target();
        if target != 0 {
            Some(target)
        } else {
            None
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
