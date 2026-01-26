use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

use crate::types::{CargoParam, Profile, Spec};

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
/// Returns None if no tspec exists (plain cargo build will be used)
pub fn find_tspec(crate_dir: &Path, explicit: Option<&str>) -> Result<Option<PathBuf>> {
    match explicit {
        Some(path) => {
            // Try as absolute or relative path
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(Some(p));
            }
            // Try relative to crate directory (e.g., -t tspec-expr.toml)
            let in_crate = crate_dir.join(path);
            if in_crate.exists() {
                return Ok(Some(in_crate));
            }
            bail!("tspec not found: {}", path);
        }
        None => {
            let default = crate_dir.join("tspec.toml");
            if default.exists() {
                Ok(Some(default))
            } else {
                Ok(None) // No tspec = plain cargo build
            }
        }
    }
}

/// Get binary path for a build with a spec
pub fn get_binary_path(workspace: &Path, crate_name: &str, spec: &Spec, release: bool) -> PathBuf {
    let is_release = release
        || spec
            .cargo
            .iter()
            .any(|p| matches!(p, CargoParam::Profile(Profile::Release)));

    let target = spec.cargo.iter().find_map(|p| match p {
        CargoParam::TargetTriple(t) => Some(t.clone()),
        CargoParam::TargetJson(p) => p.file_stem().map(|s| s.to_string_lossy().to_string()),
        _ => None,
    });

    let profile_dir = if is_release { "release" } else { "debug" };

    match target {
        Some(t) => workspace
            .join("target")
            .join(t)
            .join(profile_dir)
            .join(crate_name),
        None => workspace.join("target").join(profile_dir).join(crate_name),
    }
}

/// Get binary path for a simple build (no spec)
pub fn get_binary_path_simple(workspace: &Path, crate_name: &str, release: bool) -> PathBuf {
    let profile_dir = if release { "release" } else { "debug" };
    workspace.join("target").join(profile_dir).join(crate_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn get_binary_path_simple_debug() {
        let workspace = Path::new("/workspace");
        let path = get_binary_path_simple(workspace, "myapp", false);
        assert_eq!(path, PathBuf::from("/workspace/target/debug/myapp"));
    }

    #[test]
    fn get_binary_path_simple_release() {
        let workspace = Path::new("/workspace");
        let path = get_binary_path_simple(workspace, "myapp", true);
        assert_eq!(path, PathBuf::from("/workspace/target/release/myapp"));
    }

    #[test]
    fn get_binary_path_empty_spec_debug() {
        let workspace = Path::new("/workspace");
        let spec = Spec::default();
        let path = get_binary_path(workspace, "myapp", &spec, false);
        assert_eq!(path, PathBuf::from("/workspace/target/debug/myapp"));
    }

    #[test]
    fn get_binary_path_empty_spec_release_flag() {
        let workspace = Path::new("/workspace");
        let spec = Spec::default();
        let path = get_binary_path(workspace, "myapp", &spec, true);
        assert_eq!(path, PathBuf::from("/workspace/target/release/myapp"));
    }

    #[test]
    fn get_binary_path_release_from_spec_profile() {
        let workspace = Path::new("/workspace");
        let spec = Spec {
            cargo: vec![CargoParam::Profile(Profile::Release)],
            ..Default::default()
        };
        // release=false but spec says Release
        let path = get_binary_path(workspace, "myapp", &spec, false);
        assert_eq!(path, PathBuf::from("/workspace/target/release/myapp"));
    }

    #[test]
    fn get_binary_path_with_target_triple() {
        let workspace = Path::new("/workspace");
        let spec = Spec {
            cargo: vec![CargoParam::TargetTriple(
                "x86_64-unknown-linux-musl".to_string(),
            )],
            ..Default::default()
        };
        let path = get_binary_path(workspace, "myapp", &spec, true);
        assert_eq!(
            path,
            PathBuf::from("/workspace/target/x86_64-unknown-linux-musl/release/myapp")
        );
    }

    #[test]
    fn get_binary_path_with_target_json() {
        let workspace = Path::new("/workspace");
        let spec = Spec {
            cargo: vec![CargoParam::TargetJson(PathBuf::from(
                "x86_64-unknown-linux-rlibcx2.json",
            ))],
            ..Default::default()
        };
        let path = get_binary_path(workspace, "myapp", &spec, true);
        assert_eq!(
            path,
            PathBuf::from("/workspace/target/x86_64-unknown-linux-rlibcx2/release/myapp")
        );
    }

    #[test]
    fn get_binary_path_target_triple_debug() {
        let workspace = Path::new("/workspace");
        let spec = Spec {
            cargo: vec![CargoParam::TargetTriple(
                "x86_64-unknown-linux-musl".to_string(),
            )],
            ..Default::default()
        };
        let path = get_binary_path(workspace, "myapp", &spec, false);
        assert_eq!(
            path,
            PathBuf::from("/workspace/target/x86_64-unknown-linux-musl/debug/myapp")
        );
    }

    #[test]
    fn get_binary_path_release_flag_with_debug_spec() {
        let workspace = Path::new("/workspace");
        let spec = Spec {
            cargo: vec![CargoParam::Profile(Profile::Debug)],
            ..Default::default()
        };
        // release=true overrides spec's Debug profile
        let path = get_binary_path(workspace, "myapp", &spec, true);
        assert_eq!(path, PathBuf::from("/workspace/target/release/myapp"));
    }
}
