//! CLI commands for tspec management (cargo xt ts ...)

use anyhow::Result;
use std::path::Path;

use crate::TSPEC_SUFFIX;
use crate::find_paths::{find_crate_dir, find_tspec, find_workspace_root};
use crate::workspace::WorkspaceInfo;

/// List all tspec files in workspace or for a specific crate
pub fn list_tspecs(crate_name: Option<&str>) -> Result<()> {
    let workspace = find_workspace_root()?;

    match crate_name {
        Some(name) => {
            let crate_dir = find_crate_dir(&workspace, name)?;
            let tspecs = find_tspec_files(&crate_dir)?;
            print_crate_tspecs(name, &crate_dir, &tspecs);
        }
        None => {
            let info = WorkspaceInfo::discover()?;
            let mut found_any = false;

            for member in &info.members {
                let tspecs = find_tspec_files(&member.path)?;
                if !tspecs.is_empty() {
                    print_crate_tspecs(&member.name, &member.path, &tspecs);
                    found_any = true;
                }
            }

            if !found_any {
                println!("No *{} files found in workspace", TSPEC_SUFFIX);
            }
        }
    }

    Ok(())
}

/// Show a tspec file's contents
pub fn show_tspec(crate_name: &str, tspec: Option<&str>) -> Result<()> {
    let workspace = find_workspace_root()?;
    let crate_dir = find_crate_dir(&workspace, crate_name)?;

    match tspec {
        Some(name) => {
            // Explicit tspec - show just that one
            let path = find_tspec(&crate_dir, Some(name))?;
            match path {
                Some(p) => print_tspec_content(&p)?,
                None => anyhow::bail!("tspec '{}' not found for crate '{}'", name, crate_name),
            }
        }
        None => {
            // No tspec specified - show all tspec files
            let tspecs = find_tspec_files(&crate_dir)?;
            if tspecs.is_empty() {
                println!("No *{} files found for {}", TSPEC_SUFFIX, crate_name);
            } else {
                for (i, name) in tspecs.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    print_tspec_content(&crate_dir.join(name))?;
                }
            }
        }
    }

    Ok(())
}

/// Print a single tspec file with header
fn print_tspec_content(path: &Path) -> Result<()> {
    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    println!("====== {} ======", filename);
    let content = std::fs::read_to_string(path)?;
    print!("{}", content);
    if !content.ends_with('\n') {
        println!();
    }
    Ok(())
}

/// Find all tspec files (files ending with TSPEC_SUFFIX) in a directory
fn find_tspec_files(dir: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(TSPEC_SUFFIX) {
                files.push(name);
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Print tspec files for a crate
fn print_crate_tspecs(crate_name: &str, crate_dir: &Path, tspecs: &[String]) {
    println!("{}:", crate_name);
    for tspec in tspecs {
        let path = crate_dir.join(tspec);
        let size = std::fs::metadata(&path)
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|_| "?".to_string());
        println!("  {} ({})", tspec, size);
    }
}

/// Format file size in human-readable form
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_size_kb() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(2560), "2.5 KB");
    }
}
