//! `cargo xt ts hash` - Show tspec content hash

use anyhow::Result;
use std::path::Path;

use crate::TSPEC_SUFFIX;
use crate::find_paths::{find_tspec, find_workspace_root, get_crate_name, resolve_package_dir};
use crate::tspec::{hash_spec, load_spec};

use super::list::find_tspec_files;

/// Show hash of a tspec file
pub fn hash_tspec(package: Option<&str>, tspec: Option<&str>) -> Result<()> {
    let workspace = find_workspace_root()?;
    let package_dir = resolve_package_dir(&workspace, package)?;
    let pkg_name = get_crate_name(&package_dir)?;

    match tspec {
        Some(name) => {
            // Explicit tspec - hash just that one
            let path = find_tspec(&package_dir, Some(name))?;
            match path {
                Some(p) => print_tspec_hash(&p)?,
                None => anyhow::bail!("tspec '{}' not found for package '{}'", name, pkg_name),
            }
        }
        None => {
            // No tspec specified - hash all tspec files
            let tspecs = find_tspec_files(&package_dir)?;
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
