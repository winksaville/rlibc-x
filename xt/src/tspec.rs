use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

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
}
