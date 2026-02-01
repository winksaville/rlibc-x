//! `cargo xt ts show` - Show tspec contents

use anyhow::Result;
use std::path::Path;

use crate::TSPEC_SUFFIX;
use crate::find_paths::{find_tspec, find_workspace_root, get_crate_name, resolve_package_dir};

use super::list::find_tspec_files;

/// Show a tspec file's contents
pub fn show_tspec(package: Option<&str>, all: bool, tspec: Option<&str>) -> Result<()> {
    let workspace = find_workspace_root()?;

    // Check if we're in a package directory
    let cwd = std::env::current_dir()?;
    let in_package_dir = get_crate_name(&cwd).is_ok();

    // Resolve: --all > -p PKG > cwd > all
    let show_all = all || (package.is_none() && !in_package_dir);

    if let Some(name) = package {
        // Explicit package specified
        let package_dir = resolve_package_dir(&workspace, Some(name))?;
        show_package_tspecs(&package_dir, name, tspec)?;
    } else if show_all {
        // Show all packages
        let info = crate::workspace::WorkspaceInfo::discover()?;
        for (i, member) in info.members.iter().enumerate() {
            if i > 0 {
                println!();
            }
            println!("=== {} ===", member.name);
            show_package_tspecs(&member.path, &member.name, tspec)?;
        }
    } else {
        // In a package directory
        let pkg_name = get_crate_name(&cwd)?;
        show_package_tspecs(&cwd, &pkg_name, tspec)?;
    }

    Ok(())
}

/// Show tspecs for a single package
fn show_package_tspecs(
    package_dir: &std::path::Path,
    pkg_name: &str,
    tspec: Option<&str>,
) -> Result<()> {
    match tspec {
        Some(name) => {
            // Explicit tspec - show just that one
            let path = find_tspec(package_dir, Some(name))?;
            match path {
                Some(p) => print_tspec_content(&p)?,
                None => anyhow::bail!("tspec '{}' not found for package '{}'", name, pkg_name),
            }
        }
        None => {
            // No tspec specified - show all tspec files
            let tspecs = find_tspec_files(package_dir)?;
            if tspecs.is_empty() {
                println!("No *{} files found for {}", TSPEC_SUFFIX, pkg_name);
            } else {
                for (i, name) in tspecs.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    print_tspec_content(&package_dir.join(name))?;
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
