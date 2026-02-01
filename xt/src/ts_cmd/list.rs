//! `cargo xt ts list` - List tspec files

use anyhow::Result;
use std::path::Path;

use crate::TSPEC_SUFFIX;
use crate::find_paths::{find_workspace_root, get_crate_name, resolve_package_dir};
use crate::workspace::WorkspaceInfo;

/// List all tspec files in workspace or for a specific package
pub fn list_tspecs(package: Option<&str>, all: bool) -> Result<()> {
    let workspace = find_workspace_root()?;

    // Check if we're in a package directory (has Cargo.toml with [package], not just workspace)
    let cwd = std::env::current_dir()?;
    let in_package_dir = get_crate_name(&cwd).is_ok();

    // Resolve: --all > -p PKG > cwd > all
    let list_all = all || (package.is_none() && !in_package_dir);

    if let Some(name) = package {
        // Explicit package specified
        let package_dir = resolve_package_dir(&workspace, Some(name))?;
        let tspecs = find_tspec_files(&package_dir)?;
        print_package_tspecs(name, &package_dir, &tspecs);
    } else if list_all {
        // List all packages
        let info = WorkspaceInfo::discover()?;
        let mut found_any = false;

        for member in &info.members {
            let tspecs = find_tspec_files(&member.path)?;
            if !tspecs.is_empty() {
                print_package_tspecs(&member.name, &member.path, &tspecs);
                found_any = true;
            }
        }

        if !found_any {
            println!("No *{} files found in workspace", TSPEC_SUFFIX);
        }
    } else {
        // In a package directory, list just this package
        let pkg_name = get_crate_name(&cwd)?;
        let tspecs = find_tspec_files(&cwd)?;
        print_package_tspecs(&pkg_name, &cwd, &tspecs);
    }

    Ok(())
}

/// Find all tspec files (files ending with TSPEC_SUFFIX) in a directory
pub(crate) fn find_tspec_files(dir: &Path) -> Result<Vec<String>> {
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

/// Print tspec files for a package
fn print_package_tspecs(package_name: &str, package_dir: &Path, tspecs: &[String]) {
    println!("{}:", package_name);
    for tspec in tspecs {
        let path = package_dir.join(tspec);
        let size = std::fs::metadata(&path)
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|_| "?".to_string());
        println!("  {} ({})", tspec, size);
    }
}

/// Format file size in human-readable form
pub(crate) fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_constants::SUFFIX;
    use tempfile::TempDir;

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

    #[test]
    fn find_tspec_files_empty_dir() {
        let dir = TempDir::new().unwrap();
        let files = find_tspec_files(dir.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn find_tspec_files_returns_sorted() {
        let dir = TempDir::new().unwrap();

        // Create files in non-alphabetical order
        std::fs::write(dir.path().join(format!("zebra{}", SUFFIX)), "").unwrap();
        std::fs::write(dir.path().join(format!("alpha{}", SUFFIX)), "").unwrap();
        std::fs::write(dir.path().join(format!("middle{}", SUFFIX)), "").unwrap();

        let files = find_tspec_files(dir.path()).unwrap();

        assert_eq!(files.len(), 3);
        assert_eq!(files[0], format!("alpha{}", SUFFIX));
        assert_eq!(files[1], format!("middle{}", SUFFIX));
        assert_eq!(files[2], format!("zebra{}", SUFFIX));
    }

    #[test]
    fn find_tspec_files_filters_by_suffix() {
        let dir = TempDir::new().unwrap();

        // Create mix of tspec and non-tspec files
        std::fs::write(dir.path().join(format!("tspec{}", SUFFIX)), "").unwrap();
        std::fs::write(dir.path().join("other.toml"), "").unwrap();
        std::fs::write(dir.path().join("readme.md"), "").unwrap();
        std::fs::write(dir.path().join(format!("opt{}", SUFFIX)), "").unwrap();

        let files = find_tspec_files(dir.path()).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.contains(&format!("opt{}", SUFFIX)));
        assert!(files.contains(&format!("tspec{}", SUFFIX)));
    }

    #[test]
    fn find_tspec_files_nonexistent_dir() {
        let files = find_tspec_files(Path::new("/nonexistent/path")).unwrap();
        assert!(files.is_empty());
    }
}
