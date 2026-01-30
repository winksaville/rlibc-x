use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

use xt::all::{build_all, print_run_summary, print_summary, print_test_summary, run_all, test_all};
use xt::binary::strip_binary;
use xt::cargo_build::build_crate;
use xt::cli::{Cli, Commands, TspecCommands};
use xt::compare::compare_specs;
use xt::find_paths::{find_crate_dir, find_tspecs, find_workspace_root};
use xt::run::run_binary;
use xt::testing::test_crate;
use xt::workspace::WorkspaceInfo;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            crate_name,
            tspec,
            release,
            strip,
            fail_fast,
        } => match crate_name {
            None => {
                // Build all crates
                let workspace = WorkspaceInfo::discover()?;
                let results = build_all(&workspace, tspec.as_deref(), release, strip, fail_fast);
                return Ok(print_summary(&results));
            }
            Some(name) => {
                let result = build_crate(&name, tspec.as_deref(), release)?;
                if strip {
                    strip_binary(&result.binary_path)?;
                }
            }
        },
        Commands::Run {
            crate_name,
            tspec,
            release,
            strip,
        } => match crate_name {
            None => {
                // Run all apps
                let workspace = WorkspaceInfo::discover()?;
                let results = run_all(&workspace, tspec.as_deref(), release, strip);
                return Ok(print_run_summary(&results));
            }
            Some(name) => {
                // Build, optionally strip, then run
                let result = build_crate(&name, tspec.as_deref(), release)?;
                if strip {
                    strip_binary(&result.binary_path)?;
                }
                let exit_code = run_binary(&result.binary_path)?;
                std::process::exit(exit_code);
            }
        },
        Commands::Test {
            crate_name,
            tspec,
            release,
            fail_fast,
        } => match crate_name {
            None => {
                // Test all crates
                let workspace = WorkspaceInfo::discover()?;
                let results = test_all(&workspace, tspec.as_deref(), release, fail_fast);
                return Ok(print_test_summary(&results));
            }
            Some(name) => {
                test_crate(&name, tspec.as_deref(), release)?;
            }
        },
        Commands::Compare {
            crate_name,
            tspec,
            release,
            strip,
        } => {
            let workspace = find_workspace_root()?;
            let crate_dir = find_crate_dir(&workspace, &crate_name)?;
            let spec_paths = find_tspecs(&crate_dir, &tspec)?;
            compare_specs(&crate_name, &spec_paths, release, strip)?;
        }
        Commands::Compat { crate_name, spec } => {
            match spec {
                Some(s) => println!("compat add: crate={crate_name} spec={s}"),
                None => println!("compat show: crate={crate_name}"),
            }
            // TODO: implement
        }
        Commands::Incompat { crate_name, spec } => {
            println!("incompat add: crate={crate_name} spec={spec}");
            // TODO: implement
        }
        Commands::Tspec { command } => match command {
            TspecCommands::List { crate_name } => {
                xt::tspec_cmd::list_tspecs(crate_name.as_deref())?;
            }
            TspecCommands::Show { crate_name, tspec } => {
                xt::tspec_cmd::show_tspec(&crate_name, tspec.as_deref())?;
            }
            TspecCommands::Hash { crate_name, tspec } => {
                xt::tspec_cmd::hash_tspec(&crate_name, tspec.as_deref())?;
            }
        },
    }

    Ok(ExitCode::SUCCESS)
}
