use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xt")]
#[command(about = "Translation spec based build system")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build crate(s) with a translation spec
    Build {
        /// Crate to build (omit for all workspace members)
        crate_name: Option<String>,
        /// Translation spec to use (defaults to crate's tspec.xt.toml)
        #[arg(short = 't', long = "tspec")]
        tspec: Option<String>,
        /// Release build
        #[arg(short, long)]
        release: bool,
        /// Strip symbols from binary after build
        #[arg(short, long)]
        strip: bool,
        /// Stop on first failure (for all-crates mode)
        #[arg(short, long)]
        fail_fast: bool,
    },
    /// Build and run crate(s) with a translation spec
    Run {
        /// Crate to run (omit for all apps)
        crate_name: Option<String>,
        /// Translation spec to use (defaults to crate's tspec.xt.toml)
        #[arg(short = 't', long = "tspec")]
        tspec: Option<String>,
        /// Release build
        #[arg(short, long)]
        release: bool,
        /// Strip symbols from binary before running
        #[arg(short, long)]
        strip: bool,
    },
    /// Test crate(s) with a translation spec
    Test {
        /// Crate to test (omit for all workspace members)
        crate_name: Option<String>,
        /// Translation spec to use (defaults to crate's tspec.xt.toml)
        #[arg(short = 't', long = "tspec")]
        tspec: Option<String>,
        /// Release build
        #[arg(short, long)]
        release: bool,
        /// Stop on first failure
        #[arg(short, long)]
        fail_fast: bool,
    },
    /// Compare specs for a crate (size only)
    Compare {
        /// Crate to compare
        crate_name: String,
        /// Spec file(s) or glob pattern(s) (defaults to tspec*.xt.toml)
        #[arg(short = 't', long = "tspec", action = clap::ArgAction::Append)]
        tspec: Vec<String>,
        /// Release build
        #[arg(short, long)]
        release: bool,
        /// Strip symbols before comparing sizes
        #[arg(short, long)]
        strip: bool,
    },
    /// Manage crate compatibility with specs
    Compat {
        /// Crate name
        crate_name: String,
        /// Spec to add to compat list (omit to show current state)
        spec: Option<String>,
    },
    /// Mark a spec as incompatible with a crate
    Incompat {
        /// Crate name
        crate_name: String,
        /// Spec to add to incompat list
        spec: String,
    },
    /// Show or manage translation specs
    Spec {
        #[command(subcommand)]
        command: SpecCommands,
    },
}

#[derive(Subcommand)]
pub enum SpecCommands {
    /// List available global specs
    List,
    /// Show a spec (resolved with local mods if --crate specified)
    Show {
        /// Spec name
        name: String,
        /// Crate to apply local mods from
        #[arg(long = "crate")]
        crate_name: Option<String>,
    },
    /// Show the hash of a resolved spec
    Hash {
        /// Spec name
        name: String,
        /// Crate to apply local mods from
        #[arg(long = "crate")]
        crate_name: Option<String>,
    },
}
