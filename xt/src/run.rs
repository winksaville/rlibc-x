use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::find_paths::{find_crate_dir, find_tspec, find_workspace_root};
use crate::tspec::load_spec;
use crate::types::{CargoParam, Profile};

/// Build and run a crate with a spec
pub fn run_crate(crate_name: &str, tspec: Option<&str>, release: bool) -> Result<()> {
    let workspace = find_workspace_root()?;
    let crate_dir = find_crate_dir(&workspace, crate_name)?;
    let tspec_path = find_tspec(&crate_dir, tspec)?;

    // Determine profile and target for binary path
    let (is_release, target) = if let Some(path) = &tspec_path {
        let spec = load_spec(path)?;
        let is_rel = release
            || spec
                .cargo
                .iter()
                .any(|p| matches!(p, CargoParam::Profile(Profile::Release)));
        let tgt = spec.cargo.iter().find_map(|p| match p {
            CargoParam::TargetTriple(t) => Some(t.clone()),
            CargoParam::TargetJson(p) => p.file_stem().map(|s| s.to_string_lossy().to_string()),
            _ => None,
        });
        (is_rel, tgt)
    } else {
        (release, None)
    };

    // Build first
    crate::build::build_crate(crate_name, tspec, release)?;

    // Find and run binary
    let profile_dir = if is_release { "release" } else { "debug" };
    let binary: PathBuf = match &target {
        Some(t) => workspace
            .join("target")
            .join(t)
            .join(profile_dir)
            .join(crate_name),
        None => workspace.join("target").join(profile_dir).join(crate_name),
    };

    println!("Running {}", binary.display());
    let status = Command::new(&binary)
        .status()
        .with_context(|| format!("failed to run {}", binary.display()))?;

    std::process::exit(status.code().unwrap_or(1));
}
