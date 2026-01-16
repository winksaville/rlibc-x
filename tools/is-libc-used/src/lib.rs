//! Check if a binary uses libc.
//!
//! This library provides functionality to determine if an ELF binary
//! depends on libc or other C runtime libraries.

use object::elf::PT_INTERP;
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endianness, Object, ObjectSection};
use std::path::Path;

/// Result of checking whether a binary uses libc.
#[derive(Debug, Clone)]
pub struct LibcCheckResult {
    /// True if the binary uses libc (has INTERP or NEEDED libraries).
    pub uses_libc: bool,
    /// Informational messages collected during checks.
    pub info: Vec<String>,
}

/// Error type for libc checking operations.
#[derive(Debug)]
pub enum Error {
    /// Failed to read the file.
    Io(std::io::Error),
    /// Failed to parse the file as ELF.
    Parse(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Check if a binary uses libc.
///
/// This performs two authoritative checks:
/// 1. INTERP program header - indicates binary needs a dynamic linker
/// 2. NEEDED entries - indicates binary links against shared libraries
///
/// # Arguments
/// * `path` - Path to the ELF binary to check
///
/// # Returns
/// * `Ok(LibcCheckResult)` - Result indicating whether libc is used
/// * `Err(Error)` - If the file cannot be read or parsed
pub fn is_libc_used(path: &Path) -> Result<LibcCheckResult, Error> {
    let data = std::fs::read(path)?;
    is_libc_used_from_bytes(&data)
}

/// Check if binary data uses libc.
///
/// Same as `is_libc_used` but takes raw bytes instead of a path.
pub fn is_libc_used_from_bytes(data: &[u8]) -> Result<LibcCheckResult, Error> {
    let mut info = Vec::new();
    let mut uses_libc = false;

    // Parse as ELF
    let elf: ElfFile<'_, object::elf::FileHeader64<Endianness>> =
        ElfFile::parse(data).map_err(|e| Error::Parse(e.to_string()))?;

    // Check 1: INTERP program header (needs dynamic linker)
    let has_interp = check_interp(&elf, &mut info);
    if has_interp {
        uses_libc = true;
    }

    // Check 2: NEEDED libraries
    let needed_libs = check_needed(&elf, &mut info);
    if !needed_libs.is_empty() {
        uses_libc = true;
    }

    Ok(LibcCheckResult { uses_libc, info })
}

/// Check for INTERP program header.
fn check_interp(
    elf: &ElfFile<'_, object::elf::FileHeader64<Endianness>>,
    info: &mut Vec<String>,
) -> bool {
    let endian = elf.endian();

    // Get raw ELF data to check program headers
    let elf_header = elf.elf_header();
    let data = elf.data();

    // Parse program headers
    if let Ok(segments) = elf_header.program_headers(endian, data) {
        for segment in segments {
            if segment.p_type(endian) == PT_INTERP {
                // Try to read the interpreter path
                if let Ok(interp_data) = segment.data(endian, data) {
                    let interp = String::from_utf8_lossy(interp_data)
                        .trim_end_matches('\0')
                        .to_string();
                    info.push(format!("INTERP: {} (needs dynamic linker)", interp));
                } else {
                    info.push("INTERP: present (needs dynamic linker)".to_string());
                }
                return true;
            }
        }
    }

    info.push("INTERP: none (no dynamic linker needed)".to_string());
    false
}

/// Check for NEEDED entries in dynamic section.
fn check_needed(
    elf: &ElfFile<'_, object::elf::FileHeader64<Endianness>>,
    info: &mut Vec<String>,
) -> Vec<String> {
    let mut needed = Vec::new();

    // Look for .dynamic section
    for section in elf.sections() {
        if section.name() == Ok(".dynamic") {
            // Parse dynamic entries
            if let Ok(data) = section.data() {
                let endian = elf.endian();
                let entry_size = 16; // sizeof(Elf64_Dyn)

                // Find STRTAB first
                let mut strtab_addr = 0u64;
                let mut i = 0;
                while i + entry_size <= data.len() {
                    let tag = read_u64(endian, &data[i..i + 8]);
                    let val = read_u64(endian, &data[i + 8..i + 16]);
                    if tag == 5 {
                        // DT_STRTAB
                        strtab_addr = val;
                        break;
                    }
                    if tag == 0 {
                        // DT_NULL
                        break;
                    }
                    i += entry_size;
                }

                // Find string table section by address
                let strtab_data: Option<&[u8]> = elf.sections().find_map(|s| {
                    if s.address() == strtab_addr {
                        s.data().ok()
                    } else {
                        None
                    }
                });

                // Now find NEEDED entries
                let mut i = 0;
                while i + entry_size <= data.len() {
                    let tag = read_u64(endian, &data[i..i + 8]);
                    let val = read_u64(endian, &data[i + 8..i + 16]);

                    if tag == 1 {
                        // DT_NEEDED
                        let name = if let Some(strtab) = strtab_data {
                            let offset = val as usize;
                            if offset < strtab.len() {
                                let end = strtab[offset..]
                                    .iter()
                                    .position(|&b| b == 0)
                                    .unwrap_or(strtab.len() - offset);
                                String::from_utf8_lossy(&strtab[offset..offset + end]).to_string()
                            } else {
                                format!("<offset {}>", val)
                            }
                        } else {
                            format!("<offset {}>", val)
                        };
                        needed.push(name);
                    }

                    if tag == 0 {
                        // DT_NULL
                        break;
                    }
                    i += entry_size;
                }
            }
            break;
        }
    }

    if needed.is_empty() {
        info.push("NEEDED: none".to_string());
    } else {
        info.push(format!("NEEDED: {}", needed.join(", ")));
    }

    needed
}

fn read_u64(endian: Endianness, data: &[u8]) -> u64 {
    let bytes: [u8; 8] = data[..8].try_into().unwrap();
    match endian {
        Endianness::Little => u64::from_le_bytes(bytes),
        Endianness::Big => u64::from_be_bytes(bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_struct() {
        let result = LibcCheckResult {
            uses_libc: false,
            info: vec!["test".to_string()],
        };
        assert!(!result.uses_libc);
        assert_eq!(result.info.len(), 1);
    }
}
