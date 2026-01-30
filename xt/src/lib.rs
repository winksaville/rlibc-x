pub mod all;
pub mod binary;
pub mod cargo_build;
pub mod cli;
pub mod compare;
pub mod find_paths;
pub mod options;
pub mod print_header;
pub mod print_hline;
pub mod run;
pub mod testing;
pub mod tspec;
pub mod tspec_cmd;
pub mod types;
pub mod workspace;

/// File suffix for tspec files (e.g., "tspec.xt.toml")
pub const TSPEC_SUFFIX: &str = ".xt.toml";

#[cfg(test)]
pub mod test_constants {
    /// Test version of TSPEC_SUFFIX - kept separate so tests break if main constant changes
    pub const SUFFIX: &str = ".xt.toml";
}
