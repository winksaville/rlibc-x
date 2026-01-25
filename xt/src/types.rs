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
#[serde(rename_all = "kebab-case")]
pub enum PanicStrategy {
    Abort,
    Unwind,
    /// Nightly only: -C panic=immediate-abort (eliminates panic formatting)
    ImmediateAbort,
}

/// Cargo-specific parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoParam {
    Profile(Profile),
    TargetTriple(String),
    TargetJson(PathBuf),
    /// Nightly-only -Z flag (e.g., "panic-immediate-abort")
    Unstable(String),
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

/// Version script configuration for symbol visibility
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionScript {
    /// Symbols to keep global (e.g., ["_start"])
    pub global: Vec<String>,
    /// Pattern for local symbols (typically "*")
    #[serde(default = "default_local")]
    pub local: String,
}

fn default_local() -> String {
    "*".to_string()
}

/// Linker parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkerParam {
    /// Linker arguments from tspec.toml
    Args(Vec<String>),
    /// Version script for symbol visibility (enables --gc-sections optimization)
    VersionScript(VersionScript),
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
