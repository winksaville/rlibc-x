use anyhow::Result;
use clap::Parser;

use xt::build::build_crate;
use xt::cli::{Cli, Commands, SpecCommands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            crate_name,
            tspec,
            release,
        } => {
            build_crate(&crate_name, tspec.as_deref(), release)?;
        }
        Commands::Run {
            crate_name,
            tspec,
            release,
        } => {
            // TODO: build then run
            println!("run: crate={crate_name} tspec={tspec:?} release={release}");
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
