use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::options::{PanicMode, StripMode};

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

/// Cargo-specific configuration (flat struct)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoConfig {
    /// Build profile (debug or release)
    pub profile: Option<Profile>,
    /// Target triple (e.g., "x86_64-unknown-linux-musl")
    pub target_triple: Option<String>,
    /// Custom target JSON file path
    pub target_json: Option<PathBuf>,
    /// Nightly-only -Z flags (e.g., ["panic-immediate-abort"])
    #[serde(default)]
    pub unstable: Vec<String>,
}

/// Rustc codegen and compilation configuration (flat struct)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcConfig {
    /// Optimization level (-C opt-level=)
    pub opt_level: Option<OptLevel>,
    /// Panic strategy (-C panic=)
    pub panic: Option<PanicStrategy>,
    /// Enable LTO (-C lto=true)
    pub lto: Option<bool>,
    /// Codegen units (-C codegen-units=)
    pub codegen_units: Option<u32>,
    /// Crates to rebuild with -Z build-std (nightly only)
    #[serde(default)]
    pub build_std: Vec<String>,
    /// Raw flags passed through
    #[serde(default)]
    pub flags: Vec<String>,
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

/// Linker configuration (flat struct)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkerConfig {
    /// Linker arguments (e.g., ["-static", "-nostdlib"])
    #[serde(default)]
    pub args: Vec<String>,
    /// Version script for symbol visibility (enables --gc-sections optimization)
    pub version_script: Option<VersionScript>,
}

/// A translation spec
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
    /// High-level panic mode (sets both cargo -Z and rustc -C flags)
    pub panic: Option<PanicMode>,

    /// High-level strip mode (sets rustc -C strip=)
    pub strip: Option<StripMode>,

    #[serde(default)]
    pub cargo: CargoConfig,
    #[serde(default)]
    pub rustc: RustcConfig,
    #[serde(default)]
    pub linker: LinkerConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_default_is_empty() {
        let spec = Spec::default();
        assert_eq!(spec.cargo, CargoConfig::default());
        assert_eq!(spec.rustc, RustcConfig::default());
        assert_eq!(spec.linker, LinkerConfig::default());
    }
}
