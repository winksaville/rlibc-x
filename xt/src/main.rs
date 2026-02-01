use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

use xt::all::{build_all, print_run_summary, print_summary, print_test_summary, run_all, test_all};
use xt::binary::strip_binary;
use xt::cargo_build::build_crate;
use xt::cli::{Cli, Commands, TspecCommands};
use xt::compare::compare_specs;
use xt::find_paths::{find_package_dir, find_tspecs, find_workspace_root, get_crate_name};
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

/// Check if current directory is a package (has Cargo.toml with [package])
fn current_package_name() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    get_crate_name(&cwd).ok()
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            package,
            all,
            tspec,
            release,
            strip,
            fail_fast,
        } => {
            // Resolve package: --all > -p PKG > cwd > all
            let resolved = if all {
                None
            } else {
                package.or_else(current_package_name)
            };
            match resolved {
                None => {
                    // Build all packages
                    let workspace = WorkspaceInfo::discover()?;
                    let results =
                        build_all(&workspace, tspec.as_deref(), release, strip, fail_fast);
                    return Ok(print_summary(&results));
                }
                Some(name) => {
                    let result = build_crate(&name, tspec.as_deref(), release)?;
                    if strip {
                        strip_binary(&result.binary_path)?;
                    }
                }
            }
        }
        Commands::Run {
            package,
            all,
            tspec,
            release,
            strip,
        } => {
            // Resolve package: --all > -p PKG > cwd > all
            let resolved = if all {
                None
            } else {
                package.or_else(current_package_name)
            };
            match resolved {
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
            }
        }
        Commands::Test {
            package,
            all,
            tspec,
            release,
            fail_fast,
        } => {
            // Resolve package: --all > -p PKG > cwd > all
            let resolved = if all {
                None
            } else {
                package.or_else(current_package_name)
            };
            match resolved {
                None => {
                    // Test all packages
                    let workspace = WorkspaceInfo::discover()?;
                    let results = test_all(&workspace, tspec.as_deref(), release, fail_fast);
                    return Ok(print_test_summary(&results));
                }
                Some(name) => {
                    test_crate(&name, tspec.as_deref(), release)?;
                }
            }
        }
        Commands::Compare {
            package,
            tspec,
            release,
            strip,
        } => {
            let workspace = find_workspace_root()?;
            let package_dir = find_package_dir(&workspace, &package)?;
            let spec_paths = find_tspecs(&package_dir, &tspec)?;
            compare_specs(&package, &spec_paths, release, strip)?;
        }
        Commands::Compat { package, spec } => {
            match spec {
                Some(s) => println!("compat add: package={package} spec={s}"),
                None => println!("compat show: package={package}"),
            }
            // TODO: implement
        }
        Commands::Incompat { package, spec } => {
            println!("incompat add: package={package} spec={spec}");
            // TODO: implement
        }
        Commands::Tspec { command } => match command {
            TspecCommands::List { package, all } => {
                xt::ts_cmd::list_tspecs(package.as_deref(), all)?;
            }
            TspecCommands::Show {
                package,
                all,
                tspec,
            } => {
                xt::ts_cmd::show_tspec(package.as_deref(), all, tspec.as_deref())?;
            }
            TspecCommands::Hash {
                package,
                all,
                tspec,
            } => {
                xt::ts_cmd::hash_tspec(package.as_deref(), all, tspec.as_deref())?;
            }
            TspecCommands::New {
                name,
                package,
                from,
            } => {
                xt::ts_cmd::new_tspec(package.as_deref(), &name, from.as_deref())?;
            }
            TspecCommands::Set {
                assignment,
                package,
                tspec,
            } => {
                let (key, value) = assignment.split_once('=').ok_or_else(|| {
                    anyhow::anyhow!("invalid assignment '{}': expected key=value", assignment)
                })?;
                xt::ts_cmd::set_value(package.as_deref(), key, value, tspec.as_deref())?;
            }
        },
    }

    Ok(ExitCode::SUCCESS)
}
