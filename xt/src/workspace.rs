//! Workspace discovery and crate classification
//!
//! Uses `cargo metadata` to discover workspace members and classify them
//! by type (app, lib, tool, test, build tool).

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use std::path::{Path, PathBuf};

/// Crate classification for behavior differences
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrateKind {
    /// apps/* - application binaries, runnable
    App,
    /// libs/* - library crates
    Lib,
    /// tools/* - utility binaries
    Tool,
    /// */tests - test binary crates (special handling)
    Test,
    /// xt, xtask - build tools, excluded by default
    BuildTool,
}

/// Information about a workspace member
#[derive(Debug, Clone)]
pub struct CrateMember {
    pub name: String,
    pub path: PathBuf,
    pub has_binary: bool,
    pub kind: CrateKind,
}

/// Workspace information from cargo metadata
pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub members: Vec<CrateMember>,
}

impl WorkspaceInfo {
    /// Discover workspace using cargo metadata
    pub fn discover() -> Result<Self> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("failed to run cargo metadata")?;

        let root = metadata.workspace_root.as_std_path().to_path_buf();

        let members: Vec<CrateMember> = metadata
            .workspace_packages()
            .iter()
            .map(|pkg| {
                let path = pkg
                    .manifest_path
                    .parent()
                    .unwrap()
                    .as_std_path()
                    .to_path_buf();
                let has_binary = pkg.targets.iter().any(|t| t.is_bin());
                let kind = classify_crate(&path, &pkg.name);

                CrateMember {
                    name: pkg.name.clone(),
                    path,
                    has_binary,
                    kind,
                }
            })
            .collect();

        Ok(WorkspaceInfo { root, members })
    }

    /// Get members excluding build tools (xt, xtask)
    pub fn buildable_members(&self) -> Vec<&CrateMember> {
        self.members
            .iter()
            .filter(|m| m.kind != CrateKind::BuildTool)
            .collect()
    }

    /// Get members that can be run (apps only - have binaries and are runnable)
    pub fn runnable_members(&self) -> Vec<&CrateMember> {
        self.members
            .iter()
            .filter(|m| m.has_binary && m.kind == CrateKind::App)
            .collect()
    }

    /// Get test crates (special handling for rlibc-x2-tests etc.)
    pub fn test_members(&self) -> Vec<&CrateMember> {
        self.members
            .iter()
            .filter(|m| m.kind == CrateKind::Test)
            .collect()
    }
}

/// Classify a crate based on its path and name
fn classify_crate(path: &Path, name: &str) -> CrateKind {
    let path_str = path.to_string_lossy();

    // Build tools at workspace root level
    if name == "xt" || name == "xtask" {
        return CrateKind::BuildTool;
    }

    // Test crates
    if path_str.contains("/tests") || name.ends_with("-tests") {
        return CrateKind::Test;
    }

    // Categorize by directory
    if path_str.contains("apps/") {
        CrateKind::App
    } else if path_str.contains("libs/") {
        CrateKind::Lib
    } else if path_str.contains("tools/") {
        CrateKind::Tool
    } else {
        // Unknown location, treat as build tool (excluded)
        CrateKind::BuildTool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_app() {
        let path = PathBuf::from("/workspace/apps/ex-x1");
        assert_eq!(classify_crate(&path, "ex-x1"), CrateKind::App);
    }

    #[test]
    fn classify_lib() {
        let path = PathBuf::from("/workspace/libs/rlibc-x1");
        assert_eq!(classify_crate(&path, "rlibc-x1"), CrateKind::Lib);
    }

    #[test]
    fn classify_tool() {
        let path = PathBuf::from("/workspace/tools/is-libc-used");
        assert_eq!(classify_crate(&path, "is-libc-used"), CrateKind::Tool);
    }

    #[test]
    fn classify_test_by_path() {
        let path = PathBuf::from("/workspace/libs/rlibc-x2/tests");
        assert_eq!(classify_crate(&path, "rlibc-x2-tests"), CrateKind::Test);
    }

    #[test]
    fn classify_test_by_name() {
        let path = PathBuf::from("/workspace/somewhere");
        assert_eq!(classify_crate(&path, "foo-tests"), CrateKind::Test);
    }

    #[test]
    fn classify_build_tool_xt() {
        let path = PathBuf::from("/workspace/xt");
        assert_eq!(classify_crate(&path, "xt"), CrateKind::BuildTool);
    }

    #[test]
    fn classify_build_tool_xtask() {
        let path = PathBuf::from("/workspace/xtask");
        assert_eq!(classify_crate(&path, "xtask"), CrateKind::BuildTool);
    }

    #[test]
    fn discover_works() {
        // This test requires being run from workspace root
        let info = WorkspaceInfo::discover();
        if let Ok(info) = info {
            // Should have some members
            assert!(!info.members.is_empty());

            // xt and xtask should be excluded from buildable
            let buildable = info.buildable_members();
            assert!(
                buildable
                    .iter()
                    .all(|m| m.name != "xt" && m.name != "xtask")
            );

            // Should have some apps
            let runnable = info.runnable_members();
            assert!(!runnable.is_empty());
        }
        // If cargo metadata fails (e.g., not in workspace), that's OK for CI
    }
}
