use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::types::Spec;

/// Load a spec from a TOML file
pub fn load_spec(path: &Path) -> Result<Spec> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read spec file: {}", path.display()))?;
    parse_spec(&content)
}

/// Parse a spec from TOML string
pub fn parse_spec(toml_str: &str) -> Result<Spec> {
    toml::from_str(toml_str).context("failed to parse spec TOML")
}

/// Serialize a spec to TOML string (canonical form for hashing)
pub fn serialize_spec(spec: &Spec) -> Result<String> {
    toml::to_string(spec).context("failed to serialize spec")
}

/// Compute hash of a resolved spec (first 8 hex chars)
pub fn hash_spec(spec: &Spec) -> Result<String> {
    let toml_str = serialize_spec(spec)?;
    let mut hasher = Sha256::new();
    hasher.update(toml_str.as_bytes());
    let result = hasher.finalize();
    Ok(hex::encode(&result[..4]))
}

/// Save a spec to a TOML file, creating parent directories if needed
pub fn save_spec(spec: &Spec, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory: {}", parent.display()))?;
    }
    let content = serialize_spec(spec)?;
    std::fs::write(path, &content)
        .with_context(|| format!("failed to write spec file: {}", path.display()))
}

/// Save a spec snapshot with sequence number and hash: `{name}-{seq:03}-{hash}.toml`
/// Returns the path of the created file.
pub fn save_spec_snapshot(spec: &Spec, name: &str, dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(dir)
        .with_context(|| format!("failed to create directory: {}", dir.display()))?;

    // Find highest existing sequence number for this name
    let prefix = format!("{}-", name);
    let next_seq = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read directory: {}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let filename = entry.file_name().to_string_lossy().into_owned();
            if filename.starts_with(&prefix) && filename.ends_with(".toml") {
                // Extract sequence number: {name}-{seq:03}-{hash}.toml
                let rest = filename.strip_prefix(&prefix)?;
                let seq_str = rest.split('-').next()?;
                seq_str.parse::<u32>().ok()
            } else {
                None
            }
        })
        .max()
        .map(|n| n + 1)
        .unwrap_or(1);

    let hash = hash_spec(spec)?;
    let filename = format!("{}-{:03}-{}.toml", name, next_seq, hash);
    let path = dir.join(&filename);

    let content = serialize_spec(spec)?;
    std::fs::write(&path, &content)
        .with_context(|| format!("failed to write spec file: {}", path.display()))?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn parse_empty_spec() {
        let spec = parse_spec("").unwrap();
        assert!(spec.cargo.is_empty());
        assert!(spec.rustc.is_empty());
        assert!(spec.linker.is_empty());
    }

    #[test]
    fn hash_is_stable() {
        let spec = Spec::default();
        let hash1 = hash_spec(&spec).unwrap();
        let hash2 = hash_spec(&spec).unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 8);
    }

    #[test]
    fn different_specs_different_hash() {
        let empty = Spec::default();
        let with_release = Spec {
            cargo: vec![CargoParam::Profile(Profile::Release)],
            ..Default::default()
        };
        assert_ne!(
            hash_spec(&empty).unwrap(),
            hash_spec(&with_release).unwrap()
        );
    }

    #[test]
    fn save_and_load_roundtrip() {
        let spec = Spec {
            cargo: vec![CargoParam::Profile(Profile::Release)],
            rustc: vec![RustcParam::Lto(true)],
            linker: vec![LinkerParam::Static],
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");

        save_spec(&spec, &path).unwrap();
        let loaded = load_spec(&path).unwrap();

        assert_eq!(spec.cargo.len(), loaded.cargo.len());
        assert_eq!(spec.rustc.len(), loaded.rustc.len());
        assert_eq!(spec.linker.len(), loaded.linker.len());
    }

    #[test]
    fn save_spec_creates_parent_dirs() {
        let spec = Spec::default();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("deep").join("test.toml");

        save_spec(&spec, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn snapshot_creates_sequenced_files() {
        let dir = tempfile::tempdir().unwrap();
        let spec1 = Spec::default();
        let spec2 = Spec {
            cargo: vec![CargoParam::Profile(Profile::Release)],
            ..Default::default()
        };

        let path1 = save_spec_snapshot(&spec1, "test", dir.path()).unwrap();
        let path2 = save_spec_snapshot(&spec2, "test", dir.path()).unwrap();
        let path3 = save_spec_snapshot(&spec1, "test", dir.path()).unwrap();

        let name1 = path1.file_name().unwrap().to_string_lossy();
        let name2 = path2.file_name().unwrap().to_string_lossy();
        let name3 = path3.file_name().unwrap().to_string_lossy();

        assert!(name1.starts_with("test-001-"));
        assert!(name2.starts_with("test-002-"));
        assert!(name3.starts_with("test-003-"));

        // Same content = same hash suffix
        let hash1 = name1.strip_prefix("test-001-").unwrap();
        let hash3 = name3.strip_prefix("test-003-").unwrap();
        assert_eq!(hash1, hash3);
    }
}
