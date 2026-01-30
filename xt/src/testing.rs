use anyhow::{Context, Result, bail};
use std::fs;
use std::process::Command;

use crate::cargo_build::{apply_spec_to_command, generate_build_rs};
use crate::find_paths::{find_crate_dir, find_tspec, find_workspace_root};
use crate::tspec::load_spec;
use crate::types::{LinkerParam, RustcParam};

/// Check if spec requires nightly toolchain
fn requires_nightly(spec: &crate::types::Spec) -> bool {
    // High-level panic mode may require nightly
    let panic_needs_nightly = spec.panic.map(|p| p.requires_nightly()).unwrap_or(false);

    let has_build_std = spec
        .rustc
        .iter()
        .any(|p| matches!(p, RustcParam::BuildStd(_)));

    let has_unstable = !spec.cargo.unstable.is_empty();

    panic_needs_nightly || has_build_std || has_unstable
}

/// Test a crate with a spec
pub fn test_crate(crate_name: &str, tspec: Option<&str>, release: bool) -> Result<()> {
    let workspace = find_workspace_root()?;
    let crate_dir = find_crate_dir(&workspace, crate_name)?;
    let tspec_path = find_tspec(&crate_dir, tspec)?;

    // Track if we generated a build.rs
    let build_rs_path = crate_dir.join("build.rs");
    let had_build_rs = build_rs_path.exists();

    // Apply spec if present, otherwise plain cargo test
    let status = if let Some(path) = &tspec_path {
        let spec = load_spec(path)?;
        println!("Testing {} with spec {}", crate_name, path.display());

        // Generate temporary build.rs for linker flags if needed
        let has_linker_args = spec
            .linker
            .iter()
            .any(|p| matches!(p, LinkerParam::Args(_)));
        if has_linker_args && !had_build_rs {
            generate_build_rs(&build_rs_path, crate_name, &spec)?;
        }

        let mut cmd = Command::new("cargo");
        if requires_nightly(&spec) {
            cmd.arg("+nightly");
        }
        cmd.arg("test");
        cmd.arg("-p").arg(crate_name);
        cmd.current_dir(&workspace);

        apply_spec_to_command(&mut cmd, &spec, &workspace, release)?;
        cmd.status().context("failed to run cargo test")?
    } else {
        println!("Testing {} (no tspec)", crate_name);
        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        cmd.arg("-p").arg(crate_name);
        cmd.current_dir(&workspace);
        if release {
            cmd.arg("--release");
        }
        cmd.status().context("failed to run cargo test")?
    };

    // Clean up generated build.rs (only if we created it)
    if !had_build_rs && build_rs_path.exists() {
        let _ = fs::remove_file(&build_rs_path);
    }

    if !status.success() {
        bail!("cargo test failed");
    }

    Ok(())
}
