//! Compare function sizes between binaries

use anyhow::{Context, Result};
use goblin::elf::Elf;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Run compare mode: compare function sizes between two binaries
pub fn run_compare(funcs_file: &Path, binary1: &Path, binary2: &Path) -> Result<()> {
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
    let name1 = binary1
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let name2 = binary2
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    // Print comparison table
    println!(
        "{:<40} {:>12} {:>12} {:>10}",
        "FUNCTION", name1, name2, "RATIO"
    );
    println!("{}", "-".repeat(76));

    let mut total1: u64 = 0;
    let mut total2: u64 = 0;
    let mut compared = 0;

    for func in &func_names {
        let size1 = sizes1.get(func).copied();
        let size2 = sizes2.get(func).copied();

        let s1_str = size1
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        let s2_str = size2
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());

        let ratio_str = match (size1, size2) {
            (Some(s1), Some(s2)) if s1 > 0 => {
                total1 += s1;
                total2 += s2;
                compared += 1;
                format!("{:.1}x", s2 as f64 / s1 as f64)
            }
            _ => "-".to_string(),
        };

        println!(
            "{:<40} {:>12} {:>12} {:>10}",
            func, s1_str, s2_str, ratio_str
        );
    }

    println!("{}", "-".repeat(76));
    println!(
        "{:<40} {:>12} {:>12} {:>10}",
        format!("TOTAL ({} functions)", compared),
        total1,
        total2,
        if total1 > 0 {
            format!("{:.1}x", total2 as f64 / total1 as f64)
        } else {
            "-".to_string()
        }
    );

    Ok(())
}

/// Extract function name -> size map from a binary
pub fn get_function_sizes(binary: &Path) -> Result<HashMap<String, u64>> {
    let binary_data =
        fs::read(binary).with_context(|| format!("Failed to read binary: {:?}", binary))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::build_test_binary;

    #[test]
    fn test_get_function_sizes() {
        let binary_path = build_test_binary();
        let sizes = get_function_sizes(&binary_path).expect("Should parse binary");

        assert!(!sizes.is_empty(), "Should find some functions");
        // ex-x2 should have a main function
        assert!(sizes.contains_key("main"), "Should have main function");
    }

    #[test]
    fn test_get_function_sizes_has_positive_sizes() {
        let binary_path = build_test_binary();
        let sizes = get_function_sizes(&binary_path).expect("Should parse binary");

        for (name, size) in &sizes {
            assert!(*size > 0, "Function {} should have positive size", name);
        }
    }
}
