//! `cargo xt ts new` - Create a new tspec file

use anyhow::Result;
use std::path::Path;

use crate::TSPEC_SUFFIX;
use crate::find_paths::{find_crate_dir, find_tspec, find_workspace_root};
use crate::tspec::{load_spec, save_spec};
use crate::types::Spec;

/// Create a new tspec file (public entry point)
pub fn new_tspec(crate_name: &str, name: &str, from: Option<&str>) -> Result<()> {
    let workspace = find_workspace_root()?;
    let crate_dir = find_crate_dir(&workspace, crate_name)?;

    // Resolve source spec if --from provided
    let source_spec = match from {
        Some(source) => {
            let source_path = resolve_source_spec(&workspace, crate_name, source)?;
            Some(load_spec(&source_path)?)
        }
        None => None,
    };

    create_tspec_file(&workspace, &crate_dir, name, source_spec.as_ref())
}

/// Resolve the source spec path from a --from argument
fn resolve_source_spec(
    workspace: &Path,
    crate_name: &str,
    source: &str,
) -> Result<std::path::PathBuf> {
    // Parse source: could be "crate/spec" or just "spec"
    let (source_crate, source_spec) = if source.contains('/') {
        let parts: Vec<&str> = source.splitn(2, '/').collect();
        (parts[0], Some(parts[1]))
    } else {
        // Same crate, just spec name
        (crate_name, Some(source))
    };

    let source_crate_dir = find_crate_dir(workspace, source_crate)?;
    find_tspec(&source_crate_dir, source_spec)?.ok_or_else(|| {
        anyhow::anyhow!(
            "source tspec '{}' not found in crate '{}'",
            source_spec.unwrap_or("tspec"),
            source_crate
        )
    })
}

/// Create a tspec file - core logic, testable
fn create_tspec_file(
    workspace: &Path,
    crate_dir: &Path,
    name: &str,
    source_spec: Option<&Spec>,
) -> Result<()> {
    let spec = source_spec.cloned().unwrap_or_default();
    let output_path = crate_dir.join(format!("{}{}", name, TSPEC_SUFFIX));

    // Check if file already exists
    if output_path.exists() {
        anyhow::bail!(
            "tspec '{}' already exists. Use a different name or delete the existing file.",
            output_path.file_name().unwrap().to_string_lossy(),
        );
    }

    // Save the spec
    save_spec(&spec, &output_path)?;

    println!(
        "Created {}",
        output_path
            .strip_prefix(workspace)
            .unwrap_or(&output_path)
            .display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_constants::SUFFIX;
    use crate::types::{CargoConfig, LinkerConfig, Profile};
    use tempfile::TempDir;

    #[test]
    fn create_empty_tspec() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path();

        create_tspec_file(dir.path(), crate_dir, "test", None).unwrap();

        let created = crate_dir.join(format!("test{}", SUFFIX));
        assert!(created.exists());

        let spec = load_spec(&created).unwrap();
        assert_eq!(spec, Spec::default());
    }

    #[test]
    fn create_tspec_from_source() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path();

        let source = Spec {
            cargo: CargoConfig {
                profile: Some(Profile::Release),
                ..Default::default()
            },
            linker: LinkerConfig {
                args: vec!["-static".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        create_tspec_file(dir.path(), crate_dir, "copy", Some(&source)).unwrap();

        let created = crate_dir.join(format!("copy{}", SUFFIX));
        assert!(created.exists());

        let loaded = load_spec(&created).unwrap();
        assert_eq!(loaded.cargo.profile, Some(Profile::Release));
        assert_eq!(loaded.linker.args, vec!["-static".to_string()]);
    }

    #[test]
    fn error_when_file_exists() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path();

        // Create existing file
        let existing = crate_dir.join(format!("existing{}", SUFFIX));
        std::fs::write(&existing, "").unwrap();

        let result = create_tspec_file(dir.path(), crate_dir, "existing", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }
}
