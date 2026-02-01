//! `cargo xt ts set` - Set a scalar value in a tspec

use anyhow::{Context, Result, bail};

use crate::find_paths::{find_tspec, find_workspace_root, resolve_package_dir};
use crate::options::{PanicMode, StripMode};
use crate::tspec::{load_spec, save_spec_snapshot};
use crate::types::{OptLevel, PanicStrategy, Profile, Spec};

/// Set a value in a tspec and save as versioned snapshot
pub fn set_value(package: Option<&str>, key: &str, value: &str, tspec: Option<&str>) -> Result<()> {
    let workspace = find_workspace_root()?;
    let package_dir = resolve_package_dir(&workspace, package)?;

    // Load existing spec or use default
    let mut spec = match find_tspec(&package_dir, tspec)? {
        Some(path) => load_spec(&path)?,
        None => Spec::default(),
    };

    // Apply the key=value change
    apply_value(&mut spec, key, value)?;

    // Determine base name for snapshot
    let base_name = match tspec {
        Some(t) => {
            // Strip suffix if present
            t.strip_suffix(crate::TSPEC_SUFFIX)
                .or_else(|| t.strip_suffix(".toml"))
                .unwrap_or(t)
                .to_string()
        }
        None => "tspec".to_string(),
    };

    // Save as versioned snapshot
    let output_path = save_spec_snapshot(&spec, &base_name, &package_dir)?;

    println!(
        "Saved {}",
        output_path
            .strip_prefix(&workspace)
            .unwrap_or(&output_path)
            .display()
    );

    Ok(())
}

/// Apply key=value to a spec
fn apply_value(spec: &mut Spec, key: &str, value: &str) -> Result<()> {
    match key {
        // Top-level high-level options
        "panic" => {
            spec.panic = Some(parse_panic_mode(value)?);
        }
        "strip" => {
            spec.strip = Some(parse_strip_mode(value)?);
        }

        // Cargo config
        "cargo.profile" => {
            spec.cargo.profile = Some(parse_profile(value)?);
        }
        "cargo.target_triple" => {
            spec.cargo.target_triple = Some(value.to_string());
        }

        // Rustc config
        "rustc.opt_level" => {
            spec.rustc.opt_level = Some(parse_opt_level(value)?);
        }
        "rustc.panic" => {
            spec.rustc.panic = Some(parse_panic_strategy(value)?);
        }
        "rustc.lto" => {
            spec.rustc.lto = Some(parse_bool(value)?);
        }
        "rustc.codegen_units" => {
            spec.rustc.codegen_units = Some(
                value
                    .parse()
                    .with_context(|| format!("invalid codegen_units: {}", value))?,
            );
        }

        _ => bail!("unknown key: {}", key),
    }

    Ok(())
}

fn parse_panic_mode(s: &str) -> Result<PanicMode> {
    match s {
        "unwind" => Ok(PanicMode::Unwind),
        "abort" => Ok(PanicMode::Abort),
        "immediate-abort" => Ok(PanicMode::ImmediateAbort),
        _ => bail!(
            "invalid panic mode: {} (expected: unwind, abort, immediate-abort)",
            s
        ),
    }
}

fn parse_strip_mode(s: &str) -> Result<StripMode> {
    match s {
        "none" => Ok(StripMode::None),
        "debuginfo" => Ok(StripMode::Debuginfo),
        "symbols" => Ok(StripMode::Symbols),
        _ => bail!(
            "invalid strip mode: {} (expected: none, debuginfo, symbols)",
            s
        ),
    }
}

fn parse_profile(s: &str) -> Result<Profile> {
    match s {
        "debug" => Ok(Profile::Debug),
        "release" => Ok(Profile::Release),
        _ => bail!("invalid profile: {} (expected: debug, release)", s),
    }
}

fn parse_opt_level(s: &str) -> Result<OptLevel> {
    match s {
        "0" => Ok(OptLevel::O0),
        "1" => Ok(OptLevel::O1),
        "2" => Ok(OptLevel::O2),
        "3" => Ok(OptLevel::O3),
        "s" => Ok(OptLevel::Os),
        "z" => Ok(OptLevel::Oz),
        _ => bail!("invalid opt-level: {} (expected: 0, 1, 2, 3, s, z)", s),
    }
}

fn parse_panic_strategy(s: &str) -> Result<PanicStrategy> {
    match s {
        "abort" => Ok(PanicStrategy::Abort),
        "unwind" => Ok(PanicStrategy::Unwind),
        "immediate-abort" => Ok(PanicStrategy::ImmediateAbort),
        _ => bail!(
            "invalid panic strategy: {} (expected: abort, unwind, immediate-abort)",
            s
        ),
    }
}

fn parse_bool(s: &str) -> Result<bool> {
    match s {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => bail!("invalid boolean: {} (expected: true/false)", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_strip_mode() {
        let mut spec = Spec::default();
        apply_value(&mut spec, "strip", "symbols").unwrap();
        assert_eq!(spec.strip, Some(StripMode::Symbols));
    }

    #[test]
    fn apply_panic_mode() {
        let mut spec = Spec::default();
        apply_value(&mut spec, "panic", "abort").unwrap();
        assert_eq!(spec.panic, Some(PanicMode::Abort));
    }

    #[test]
    fn apply_rustc_lto() {
        let mut spec = Spec::default();
        apply_value(&mut spec, "rustc.lto", "true").unwrap();
        assert_eq!(spec.rustc.lto, Some(true));
    }

    #[test]
    fn apply_rustc_opt_level() {
        let mut spec = Spec::default();
        apply_value(&mut spec, "rustc.opt_level", "z").unwrap();
        assert_eq!(spec.rustc.opt_level, Some(OptLevel::Oz));
    }

    #[test]
    fn apply_cargo_profile() {
        let mut spec = Spec::default();
        apply_value(&mut spec, "cargo.profile", "release").unwrap();
        assert_eq!(spec.cargo.profile, Some(Profile::Release));
    }

    #[test]
    fn unknown_key_errors() {
        let mut spec = Spec::default();
        let result = apply_value(&mut spec, "nonexistent", "value");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown key"));
    }

    #[test]
    fn invalid_strip_mode_errors() {
        let mut spec = Spec::default();
        let result = apply_value(&mut spec, "strip", "invalid");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid strip mode")
        );
    }

    #[test]
    fn apply_value_with_spaces() {
        let mut spec = Spec::default();
        apply_value(&mut spec, "cargo.target_triple", "my custom triple").unwrap();
        assert_eq!(
            spec.cargo.target_triple,
            Some("my custom triple".to_string())
        );
    }
}
