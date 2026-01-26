use anyhow::Result;
use clap::Parser;

use xt::binary::strip_binary;
use xt::build::build_crate;
use xt::cli::{Cli, Commands, SpecCommands};
use xt::compare::compare_specs;
use xt::run::run_binary;
use xt::testing::test_crate;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            crate_name,
            tspec,
            release,
            strip,
        } => {
            let result = build_crate(&crate_name, tspec.as_deref(), release)?;
            if strip {
                strip_binary(&result.binary_path)?;
            }
        }
        Commands::Run {
            crate_name,
            tspec,
            release,
            strip,
        } => {
            // Build, optionally strip, then run
            let result = build_crate(&crate_name, tspec.as_deref(), release)?;
            if strip {
                strip_binary(&result.binary_path)?;
            }
            let exit_code = run_binary(&result.binary_path)?;
            std::process::exit(exit_code);
        }
        Commands::Test {
            crate_name,
            tspec,
            release,
        } => {
            test_crate(&crate_name, tspec.as_deref(), release)?;
        }
        Commands::Compare {
            crate_name,
            spec_a,
            spec_b,
            release,
        } => {
            compare_specs(&crate_name, &spec_a, &spec_b, release)?;
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
        Commands::Spec { command } => {
            match command {
                SpecCommands::List => {
                    println!("spec list");
                    // TODO: implement
                }
                SpecCommands::Show { name, crate_name } => {
                    println!("spec show: name={name} crate={crate_name:?}");
                    // TODO: implement
                }
                SpecCommands::Hash { name, crate_name } => {
                    println!("spec hash: name={name} crate={crate_name:?}");
                    // TODO: implement
                }
            }
        }
    }

    Ok(())
}
