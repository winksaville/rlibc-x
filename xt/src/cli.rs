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
    /// Build a crate with a translation spec
    Build {
        /// Crate to build
        crate_name: String,
        /// Translation spec to use (defaults to crate's tspec.toml)
        #[arg(short = 't', long = "tspec")]
        tspec: Option<String>,
        /// Release build
        #[arg(short, long)]
        release: bool,
    },
    /// Build and run a crate with a translation spec
    Run {
        /// Crate to run
        crate_name: String,
        /// Translation spec to use (defaults to crate's tspec.toml)
        #[arg(short = 't', long = "tspec")]
        tspec: Option<String>,
        /// Release build
        #[arg(short, long)]
        release: bool,
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
