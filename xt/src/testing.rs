use anyhow::{Context, Result, bail};
use std::fs;
use std::process::Command;

use crate::build::{apply_spec_to_command, generate_build_rs};
use crate::find_paths::{find_crate_dir, find_tspec, find_workspace_root};
use crate::tspec::load_spec;

/// Test a crate with a spec
pub fn test_crate(crate_name: &str, tspec: Option<&str>, release: bool) -> Result<()> {
    let workspace = find_workspace_root()?;
    let crate_dir = find_crate_dir(&workspace, crate_name)?;
    let tspec_path = find_tspec(&crate_dir, tspec)?;

    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.arg("-p").arg(crate_name);
    cmd.current_dir(&workspace);

    // Track if we generated a build.rs
    let build_rs_path = crate_dir.join("build.rs");
    let had_build_rs = build_rs_path.exists();

    // Apply spec if present, otherwise plain cargo test
    if let Some(path) = &tspec_path {
        let spec = load_spec(path)?;
        println!("Testing {} with spec {}", crate_name, path.display());

        // Generate temporary build.rs for linker flags if needed
        if !spec.linker.is_empty() && !had_build_rs {
            generate_build_rs(&build_rs_path, crate_name, &spec)?;
        }

        apply_spec_to_command(&mut cmd, &spec, release)?;
    } else {
        println!("Testing {} (no tspec)", crate_name);
        if release {
            cmd.arg("--release");
        }
    }

    let status = cmd.status().context("failed to run cargo test")?;

    // Clean up generated build.rs (only if we created it)
    if !had_build_rs && build_rs_path.exists() {
        let _ = fs::remove_file(&build_rs_path);
    }

    if !status.success() {
        bail!("cargo test failed");
    }

    Ok(())
}
