/************************************************
* File: main.rs
* Author: Michal Švrček
*
* DeployTool CLI entry point and command handling
*
* ver. 0.1.0
*************************************************/

mod config;

use clap::{Parser, Subcommand};
use config::Config;
use std::{path::PathBuf, process};

#[derive(Parser)]
#[command(name = "deploy")]
#[command(version)]
#[command(about = "DeployTool - Remote deployment manager")]
struct Cli {
    /// Path to the deployment configuration
    #[arg(long, global = true, default_value = "deploy.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List configured deployment targets
    List,

    /// Show deployment target information
    Info { name: String },

    /// Preview a deployment without executing it
    Plan { name: String },
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    let config = Config::load(&cli.config)?;

    match cli.command {
        Commands::List => {
            config.list();
        }

        Commands::Info { name } => {
            let target = config.get_target(&name)?;
            target.print_info(&name);
        }

        Commands::Plan { name } => {
            let target = config.get_target(&name)?;
            target.print_plan(&name);
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
