use anyhow::{Context, Result, bail};
use glob::Pattern;
use std::path::{Path, PathBuf};

use crate::types::{CargoParam, Profile, Spec};

/// Extract crate name from Cargo.toml
pub fn get_crate_name(crate_dir: &Path) -> Result<String> {
    let cargo_toml = crate_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)
        .with_context(|| format!("failed to read {}", cargo_toml.display()))?;

    // Simple parsing - look for name = "..." in [package] section
    let mut in_package = false;
    for line in content.lines() {
        let line = line.trim();
        if line == "[package]" {
            in_package = true;
            continue;
        }
        if line.starts_with('[') {
            in_package = false;
            continue;
        }
        if in_package
            && line.starts_with("name")
            && let Some(eq_pos) = line.find('=')
        {
            let value = line[eq_pos + 1..].trim();
            let value = value.trim_matches('"').trim_matches('\'');
            return Ok(value.to_string());
        }
    }
    bail!("could not find package name in {}", cargo_toml.display());
}

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

/// Find a crate's directory - tries as path first, then searches standard locations
pub fn find_crate_dir(workspace: &Path, name: &str) -> Result<PathBuf> {
    // Try as path first (relative or absolute)
    let as_path = PathBuf::from(name);
    if as_path.join("Cargo.toml").exists() {
        return Ok(as_path.canonicalize().unwrap_or(as_path));
    }

    // Fallback: search libs/, apps/, tools/
    for prefix in ["libs", "apps", "tools"] {
        let path = workspace.join(prefix).join(name);
        if path.join("Cargo.toml").exists() {
            return Ok(path);
        }
    }

    // Special case: nested test crates like libs/rlibc-x2/tests
    for prefix in ["libs", "apps"] {
        for entry in std::fs::read_dir(workspace.join(prefix)).into_iter().flatten() {
            if let Ok(entry) = entry {
                let nested = entry.path().join("tests");
                if nested.join("Cargo.toml").exists() {
                    // Check if this is the crate we're looking for
                    if let Ok(pkg_name) = get_crate_name(&nested) {
                        if pkg_name == name {
                            return Ok(nested);
                        }
                    }
                }
            }
        }
    }

    bail!("crate '{}' not found", name);
}

/// Find the tspec for a crate - tries as path first, then relative to crate_dir
/// Returns None if no tspec exists (plain cargo build will be used)
pub fn find_tspec(crate_dir: &Path, explicit: Option<&str>) -> Result<Option<PathBuf>> {
    match explicit {
        Some(name) => {
            // Try as path first (relative to cwd or absolute)
            let as_path = PathBuf::from(name);
            if as_path.exists() {
                return Ok(Some(as_path.canonicalize().unwrap_or(as_path)));
            }

            // Fallback: relative to crate directory
            let in_crate = crate_dir.join(name);
            if in_crate.exists() {
                return Ok(Some(in_crate));
            }

            // Try with .xt.toml suffix if name has no extension
            if !name.contains('.') {
                let with_suffix = crate_dir.join(format!("{}.xt.toml", name));
                if with_suffix.exists() {
                    return Ok(Some(with_suffix));
                }
            }

            bail!("tspec not found: {}", name);
        }
        None => {
            let default = crate_dir.join("tspec.xt.toml");
            if default.exists() {
                Ok(Some(default))
            } else {
                Ok(None) // No tspec = plain cargo build
            }
        }
    }
}

/// Find multiple tspecs by glob patterns
/// If no patterns given, defaults to "tspec*.xt.toml"
/// Returns sorted list of paths, errors if none found
pub fn find_tspecs(crate_dir: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let patterns: Vec<&str> = if patterns.is_empty() {
        vec!["tspec*.xt.toml"]
    } else {
        patterns.iter().map(|s| s.as_str()).collect()
    };

    let mut results = Vec::new();

    for pattern in &patterns {
        // Try as literal path first (relative to cwd or absolute)
        let as_path = PathBuf::from(pattern);
        if as_path.exists() && as_path.is_file() {
            results.push(as_path.canonicalize().unwrap_or(as_path));
            continue;
        }

        // Try as literal path relative to crate_dir
        let in_crate = crate_dir.join(pattern);
        if in_crate.exists() && in_crate.is_file() {
            results.push(in_crate);
            continue;
        }

        // Try as glob pattern in crate_dir
        let glob_pattern = Pattern::new(pattern)
            .with_context(|| format!("invalid glob pattern: {}", pattern))?;

        let entries: Vec<_> = std::fs::read_dir(crate_dir)
            .with_context(|| format!("cannot read directory: {}", crate_dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                e.path().is_file() && glob_pattern.matches(&name)
            })
            .map(|e| e.path())
            .collect();

        results.extend(entries);
    }

    // Remove duplicates and sort
    results.sort();
    results.dedup();

    if results.is_empty() {
        let pattern_list = patterns.join(", ");
        bail!(
            "no tspec files found matching '{}' in {}",
            pattern_list,
            crate_dir.display()
        );
    }

    Ok(results)
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
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ==================== get_crate_name tests ====================

    #[test]
    fn get_crate_name_valid() {
        let tmp = TempDir::new().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        fs::write(
            &cargo_toml,
            r#"[package]
name = "my-test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        let name = get_crate_name(tmp.path()).unwrap();
        assert_eq!(name, "my-test-crate");
    }

    #[test]
    fn get_crate_name_with_single_quotes() {
        let tmp = TempDir::new().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        fs::write(
            &cargo_toml,
            "[package]\nname = 'single-quoted'\nversion = '0.1.0'\n",
        )
        .unwrap();

        let name = get_crate_name(tmp.path()).unwrap();
        assert_eq!(name, "single-quoted");
    }

    #[test]
    fn get_crate_name_missing_name() {
        let tmp = TempDir::new().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nversion = \"0.1.0\"\n").unwrap();

        let result = get_crate_name(tmp.path());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("could not find package name")
        );
    }

    #[test]
    fn get_crate_name_missing_file() {
        let tmp = TempDir::new().unwrap();
        let result = get_crate_name(tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed to read"));
    }

    #[test]
    fn get_crate_name_name_in_other_section_ignored() {
        let tmp = TempDir::new().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        fs::write(
            &cargo_toml,
            r#"[dependencies]
name = "wrong-section"

[package]
name = "correct-name"
version = "0.1.0"
"#,
        )
        .unwrap();

        let name = get_crate_name(tmp.path()).unwrap();
        assert_eq!(name, "correct-name");
    }

    // ==================== find_crate_dir tests ====================

    #[test]
    fn find_crate_dir_as_path() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("my-crate");
        fs::create_dir(&crate_dir).unwrap();
        fs::write(crate_dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();

        // Pass path directly
        let found = find_crate_dir(tmp.path(), crate_dir.to_str().unwrap()).unwrap();
        assert_eq!(found.file_name().unwrap(), "my-crate");
    }

    #[test]
    fn find_crate_dir_in_libs() {
        let tmp = TempDir::new().unwrap();
        let libs_dir = tmp.path().join("libs").join("my-lib");
        fs::create_dir_all(&libs_dir).unwrap();
        fs::write(libs_dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();

        let found = find_crate_dir(tmp.path(), "my-lib").unwrap();
        assert_eq!(found, libs_dir);
    }

    #[test]
    fn find_crate_dir_in_apps() {
        let tmp = TempDir::new().unwrap();
        let apps_dir = tmp.path().join("apps").join("my-app");
        fs::create_dir_all(&apps_dir).unwrap();
        fs::write(apps_dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();

        let found = find_crate_dir(tmp.path(), "my-app").unwrap();
        assert_eq!(found, apps_dir);
    }

    #[test]
    fn find_crate_dir_libs_preferred_over_apps() {
        let tmp = TempDir::new().unwrap();
        // Create both libs/foo and apps/foo
        let libs_foo = tmp.path().join("libs").join("foo");
        let apps_foo = tmp.path().join("apps").join("foo");
        fs::create_dir_all(&libs_foo).unwrap();
        fs::create_dir_all(&apps_foo).unwrap();
        fs::write(libs_foo.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        fs::write(apps_foo.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();

        // libs/ is checked first
        let found = find_crate_dir(tmp.path(), "foo").unwrap();
        assert_eq!(found, libs_foo);
    }

    #[test]
    fn find_crate_dir_not_found() {
        let tmp = TempDir::new().unwrap();
        let result = find_crate_dir(tmp.path(), "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // ==================== find_tspec tests ====================

    #[test]
    fn find_tspec_explicit_path() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        let spec_dir = tmp.path().join("specs");
        fs::create_dir(&crate_dir).unwrap();
        fs::create_dir(&spec_dir).unwrap();

        let tspec_path = spec_dir.join("custom.toml");
        fs::write(&tspec_path, "# spec").unwrap();

        // Explicit path should be found
        let found = find_tspec(&crate_dir, Some(tspec_path.to_str().unwrap())).unwrap();
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().contains("custom.toml"));
    }

    #[test]
    fn find_tspec_explicit_relative_to_crate() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        let tspec_path = crate_dir.join("my-spec.toml");
        fs::write(&tspec_path, "# spec").unwrap();

        // Name without path, found in crate_dir
        let found = find_tspec(&crate_dir, Some("my-spec.toml")).unwrap();
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().contains("my-spec.toml"));
    }

    #[test]
    fn find_tspec_default_exists() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        let tspec_path = crate_dir.join("tspec.xt.toml");
        fs::write(&tspec_path, "# default tspec").unwrap();

        // No explicit name, should find default
        let found = find_tspec(&crate_dir, None).unwrap();
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().contains("tspec.xt.toml"));
    }

    #[test]
    fn find_tspec_default_not_exists() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        // No tspec.xt.toml, should return None (plain cargo build)
        let found = find_tspec(&crate_dir, None).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn find_tspec_explicit_not_found() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        let result = find_tspec(&crate_dir, Some("nonexistent.toml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn find_tspec_suffix_fallback() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        // Create optimized.xt.toml
        let tspec_path = crate_dir.join("optimized.xt.toml");
        fs::write(&tspec_path, "# optimized tspec").unwrap();

        // Request "optimized" without extension, should find optimized.xt.toml
        let found = find_tspec(&crate_dir, Some("optimized")).unwrap();
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().contains("optimized.xt.toml"));
    }

    #[test]
    fn find_tspec_no_suffix_fallback_when_has_extension() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        // Create foo.xt.toml but request foo.toml (has extension)
        let tspec_path = crate_dir.join("foo.xt.toml");
        fs::write(&tspec_path, "# tspec").unwrap();

        // Request "foo.toml" - should NOT try suffix fallback
        let result = find_tspec(&crate_dir, Some("foo.toml"));
        assert!(result.is_err());
    }

    // ==================== find_tspecs tests ====================

    #[test]
    fn find_tspecs_default_pattern() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        fs::write(crate_dir.join("tspec.xt.toml"), "# default").unwrap();
        fs::write(crate_dir.join("tspec-opt.xt.toml"), "# opt").unwrap();
        fs::write(crate_dir.join("other.toml"), "# other").unwrap();

        // Empty patterns = default "tspec*.xt.toml"
        let found = find_tspecs(&crate_dir, &[]).unwrap();
        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|p| p.file_name().unwrap() == "tspec.xt.toml"));
        assert!(found.iter().any(|p| p.file_name().unwrap() == "tspec-opt.xt.toml"));
    }

    #[test]
    fn find_tspecs_explicit_glob() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        fs::write(crate_dir.join("tspec.xt.toml"), "# default").unwrap();
        fs::write(crate_dir.join("tspec-opt.xt.toml"), "# opt").unwrap();
        fs::write(crate_dir.join("other.xt.toml"), "# other").unwrap();

        let found = find_tspecs(&crate_dir, &["*-opt*".to_string()]).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].file_name().unwrap(), "tspec-opt.xt.toml");
    }

    #[test]
    fn find_tspecs_explicit_files() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        fs::write(crate_dir.join("a.toml"), "# a").unwrap();
        fs::write(crate_dir.join("b.toml"), "# b").unwrap();
        fs::write(crate_dir.join("c.toml"), "# c").unwrap();

        let found = find_tspecs(&crate_dir, &["a.toml".to_string(), "c.toml".to_string()]).unwrap();
        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|p| p.file_name().unwrap() == "a.toml"));
        assert!(found.iter().any(|p| p.file_name().unwrap() == "c.toml"));
    }

    #[test]
    fn find_tspecs_sorted_and_deduped() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        fs::write(crate_dir.join("z.toml"), "# z").unwrap();
        fs::write(crate_dir.join("a.toml"), "# a").unwrap();

        // Request same file twice via different patterns
        let found = find_tspecs(&crate_dir, &["*.toml".to_string(), "a.toml".to_string()]).unwrap();
        assert_eq!(found.len(), 2); // Deduped
        assert_eq!(found[0].file_name().unwrap(), "a.toml"); // Sorted
        assert_eq!(found[1].file_name().unwrap(), "z.toml");
    }

    #[test]
    fn find_tspecs_none_found_errors() {
        let tmp = TempDir::new().unwrap();
        let crate_dir = tmp.path().join("crate");
        fs::create_dir(&crate_dir).unwrap();

        let result = find_tspecs(&crate_dir, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no tspec files found"));
    }

    // ==================== get_binary_path tests ====================

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
