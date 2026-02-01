//! `cargo xt ts hash` - Show tspec content hash

use anyhow::Result;
use std::path::Path;

use crate::TSPEC_SUFFIX;
use crate::find_paths::{find_tspec, find_workspace_root, get_crate_name, resolve_package_dir};
use crate::tspec::{hash_spec, load_spec};

use super::list::find_tspec_files;

/// Show hash of a tspec file
pub fn hash_tspec(package: Option<&str>, all: bool, tspec: Option<&str>) -> Result<()> {
    let workspace = find_workspace_root()?;

    // Check if we're in a package directory
    let cwd = std::env::current_dir()?;
    let in_package_dir = get_crate_name(&cwd).is_ok();

    // Resolve: --all > -p PKG > cwd > all
    let hash_all = all || (package.is_none() && !in_package_dir);

    if let Some(name) = package {
        // Explicit package specified
        let package_dir = resolve_package_dir(&workspace, Some(name))?;
        hash_package_tspecs(&package_dir, name, tspec)?;
    } else if hash_all {
        // Hash all packages
        let info = crate::workspace::WorkspaceInfo::discover()?;
        for member in &info.members {
            hash_package_tspecs(&member.path, &member.name, tspec)?;
        }
    } else {
        // In a package directory
        let pkg_name = get_crate_name(&cwd)?;
        hash_package_tspecs(&cwd, &pkg_name, tspec)?;
    }

    Ok(())
}

/// Hash tspecs for a single package
fn hash_package_tspecs(
    package_dir: &std::path::Path,
    pkg_name: &str,
    tspec: Option<&str>,
) -> Result<()> {
    match tspec {
        Some(name) => {
            // Explicit tspec - hash just that one
            let path = find_tspec(package_dir, Some(name))?;
            match path {
                Some(p) => print_tspec_hash(&p)?,
                None => anyhow::bail!("tspec '{}' not found for package '{}'", name, pkg_name),
            }
        }
        None => {
            // No tspec specified - hash all tspec files
            let tspecs = find_tspec_files(package_dir)?;
            if tspecs.is_empty() {
                println!("No *{} files found for {}", TSPEC_SUFFIX, pkg_name);
            } else {
                for name in &tspecs {
                    print_tspec_hash(&package_dir.join(name))?;
                }
            }
        }
    }
    Ok(())
}

/// Print hash of a single tspec file
fn print_tspec_hash(path: &Path) -> Result<()> {
    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    let spec = load_spec(path)?;
    let hash = hash_spec(&spec)?;
    println!("{}: {}", filename, hash);
    Ok(())
}
