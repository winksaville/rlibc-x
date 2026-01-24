use anyhow::{Context, Result, bail};
use std::process::Command;

use crate::find_paths::{find_crate_dir, find_tspec, find_workspace_root};
use crate::tspec::load_spec;
use crate::types::{CargoParam, LinkerParam, Profile, RustcParam, Spec};

/// Build a crate with a spec
pub fn build_crate(crate_name: &str, tspec: Option<&str>, release: bool) -> Result<()> {
    let workspace = find_workspace_root()?;
    let crate_dir = find_crate_dir(&workspace, crate_name)?;
    let tspec_path = find_tspec(&crate_dir, tspec)?;

    let spec = load_spec(&tspec_path)?;
    println!("Building {} with spec {}", crate_name, tspec_path.display());

    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.arg("-p").arg(crate_name);
    cmd.current_dir(&workspace);

    // Apply spec parameters
    apply_spec_to_command(&mut cmd, &spec, release)?;

    let status = cmd.status().context("failed to run cargo")?;
    if !status.success() {
        bail!("cargo build failed");
    }

    Ok(())
}

/// Apply spec parameters to a cargo command
fn apply_spec_to_command(cmd: &mut Command, spec: &Spec, release: bool) -> Result<()> {
    // Handle cargo params
    let mut has_profile = false;
    for param in &spec.cargo {
        match param {
            CargoParam::Profile(p) => {
                has_profile = true;
                match p {
                    Profile::Release => {
                        cmd.arg("--release");
                    }
                    Profile::Debug => {
                        // Debug is default, no flag needed
                    }
                }
            }
            CargoParam::TargetTriple(triple) => {
                cmd.arg("--target").arg(triple);
            }
            CargoParam::TargetJson(path) => {
                cmd.arg("--target").arg(path);
            }
        }
    }

    // If no profile in spec but release flag passed, use release
    if !has_profile && release {
        cmd.arg("--release");
    }

    // Collect rustc flags
    let mut rustc_flags: Vec<String> = Vec::new();
    for param in &spec.rustc {
        match param {
            RustcParam::OptLevel(level) => {
                let lvl = match level {
                    crate::types::OptLevel::O0 => "0",
                    crate::types::OptLevel::O1 => "1",
                    crate::types::OptLevel::O2 => "2",
                    crate::types::OptLevel::O3 => "3",
                    crate::types::OptLevel::Os => "s",
                    crate::types::OptLevel::Oz => "z",
                };
                rustc_flags.push(format!("-C opt-level={}", lvl));
            }
            RustcParam::Panic(strategy) => {
                let s = match strategy {
                    crate::types::PanicStrategy::Abort => "abort",
                    crate::types::PanicStrategy::Unwind => "unwind",
                };
                rustc_flags.push(format!("-C panic={}", s));
            }
            RustcParam::Lto(enabled) => {
                if *enabled {
                    rustc_flags.push("-C lto=true".to_string());
                }
            }
            RustcParam::CodegenUnits(n) => {
                rustc_flags.push(format!("-C codegen-units={}", n));
            }
            RustcParam::BuildStd(_crates) => {
                // TODO: handle -Z build-std (requires nightly)
            }
            RustcParam::Flag(flag) => {
                rustc_flags.push(flag.clone());
            }
        }
    }

    // Collect linker flags
    let mut link_args: Vec<String> = Vec::new();
    for param in &spec.linker {
        match param {
            LinkerParam::Static => {
                link_args.push("-static".to_string());
            }
            LinkerParam::NoStdlib => {
                link_args.push("-nostdlib".to_string());
            }
            LinkerParam::Entry(entry) => {
                link_args.push(format!("-e{}", entry));
            }
            LinkerParam::GcSections => {
                link_args.push("-Wl,--gc-sections".to_string());
            }
            LinkerParam::Args(args) => {
                link_args.extend(args.clone());
            }
        }
    }

    // Apply rustc flags
    if !rustc_flags.is_empty() {
        cmd.env("RUSTFLAGS", rustc_flags.join(" "));
    }

    // Apply linker args via RUSTFLAGS
    if !link_args.is_empty() {
        let existing = cmd
            .get_envs()
            .find(|(k, _)| *k == "RUSTFLAGS")
            .and_then(|(_, v)| v)
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let link_flags: Vec<String> = link_args
            .iter()
            .map(|a| format!("-C link-arg={}", a))
            .collect();

        let combined = if existing.is_empty() {
            link_flags.join(" ")
        } else {
            format!("{} {}", existing, link_flags.join(" "))
        };
        cmd.env("RUSTFLAGS", combined);
    }

    Ok(())
}
