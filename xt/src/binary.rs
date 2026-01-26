use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Strip symbols from a binary
pub fn strip_binary(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("binary not found: {}", path.display());
    }

    let status = Command::new("strip")
        .arg(path)
        .status()
        .context("failed to run strip")?;

    if !status.success() {
        bail!("strip failed");
    }

    // Report new size
    if let Ok(meta) = fs::metadata(path) {
        println!("  stripped: {} bytes", meta.len());
    }

    Ok(())
}

/// Get the size of a binary in bytes
pub fn binary_size(path: &Path) -> Result<u64> {
    let meta = fs::metadata(path)
        .with_context(|| format!("failed to get metadata for {}", path.display()))?;
    Ok(meta.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn binary_size_returns_correct_size() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_file");
        let content = b"hello world"; // 11 bytes
        std::fs::write(&path, content).unwrap();

        let size = binary_size(&path).unwrap();
        assert_eq!(size, 11);
    }

    #[test]
    fn binary_size_error_on_missing_file() {
        let path = Path::new("/nonexistent/path/to/binary");
        let result = binary_size(path);
        assert!(result.is_err());
    }

    #[test]
    fn strip_binary_error_on_missing_file() {
        let path = Path::new("/nonexistent/path/to/binary");
        let result = strip_binary(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("binary not found"));
    }

    #[test]
    fn binary_size_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty_file");
        std::fs::File::create(&path).unwrap();

        let size = binary_size(&path).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn binary_size_larger_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("larger_file");
        let mut file = std::fs::File::create(&path).unwrap();
        // Write 1000 bytes
        file.write_all(&[0u8; 1000]).unwrap();

        let size = binary_size(&path).unwrap();
        assert_eq!(size, 1000);
    }
}
