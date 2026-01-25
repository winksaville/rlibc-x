use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Build profile - mutually exclusive
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Debug,
    Release,
}

/// Optimization level - mutually exclusive
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptLevel {
    #[serde(rename = "0")]
    O0,
    #[serde(rename = "1")]
    O1,
    #[serde(rename = "2")]
    O2,
    #[serde(rename = "3")]
    O3,
    #[serde(rename = "s")]
    Os,
    #[serde(rename = "z")]
    Oz,
}

/// Panic strategy - mutually exclusive
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanicStrategy {
    Abort,
    Unwind,
}

/// Cargo-specific parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoParam {
    Profile(Profile),
    TargetTriple(String),
    TargetJson(PathBuf),
}

/// Rustc codegen and compilation parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustcParam {
    OptLevel(OptLevel),
    Panic(PanicStrategy),
    Lto(bool),
    CodegenUnits(u32),
    /// Crates to rebuild with -Z build-std (nightly only)
    BuildStd(Vec<String>),
    /// Raw flag passed through
    Flag(String),
}

/// Linker parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkerParam {
    /// Linker arguments from tspec.toml
    Args(Vec<String>),
}

/// A translation spec - ordered parameter lists
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Spec {
    #[serde(default)]
    pub cargo: Vec<CargoParam>,
    #[serde(default)]
    pub rustc: Vec<RustcParam>,
    #[serde(default)]
    pub linker: Vec<LinkerParam>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_default_is_empty() {
        let spec = Spec::default();
        assert!(spec.cargo.is_empty());
        assert!(spec.rustc.is_empty());
        assert!(spec.linker.is_empty());
    }
}
