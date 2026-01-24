use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

/// Find the workspace root by looking for Cargo.toml with [workspace]
pub fn find_workspace_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            bail!("could not find workspace root");
        }
    }
}

/// Find a crate's directory by name, searching libs/ and apps/
pub fn find_crate_dir(workspace: &Path, crate_name: &str) -> Result<PathBuf> {
    // Check libs/ first, then apps/
    for prefix in ["libs", "apps"] {
        let path = workspace.join(prefix).join(crate_name);
        if path.join("Cargo.toml").exists() {
            return Ok(path);
        }
    }
    bail!("crate '{}' not found in libs/ or apps/", crate_name);
}

/// Find the tspec for a crate - either explicit path or default tspec.toml
pub fn find_tspec(crate_dir: &Path, explicit: Option<&str>) -> Result<PathBuf> {
    match explicit {
        Some(path) => {
            // Try as absolute or relative path
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
            // Try relative to crate directory (e.g., -t tspec-expr.toml)
            let in_crate = crate_dir.join(path);
            if in_crate.exists() {
                return Ok(in_crate);
            }
            bail!("tspec not found: {}", path);
        }
        None => {
            let default = crate_dir.join("tspec.toml");
            if default.exists() {
                Ok(default)
            } else {
                bail!(
                    "no tspec.toml found in {} (use -t to specify)",
                    crate_dir.display()
                );
            }
        }
    }
}
