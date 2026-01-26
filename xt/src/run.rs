use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Run a binary and return its exit code
pub fn run_binary(binary_path: &Path) -> Result<i32> {
    println!("Running {}", binary_path.display());
    let status = Command::new(binary_path)
        .status()
        .with_context(|| format!("failed to run {}", binary_path.display()))?;

    Ok(status.code().unwrap_or(1))
}
