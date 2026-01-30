//! `cargo xt ts show` - Show tspec contents

use anyhow::Result;
use std::path::Path;

use crate::TSPEC_SUFFIX;
use crate::find_paths::{find_crate_dir, find_tspec, find_workspace_root};

use super::list::find_tspec_files;

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
