/************************************************
* File: main.rs
* Author: Michal Švrček
*
* DeployTool CLI entry point and command handling
*
* ver. 0.3.0
*************************************************/

mod build;
mod config;
mod output;
mod ssh;
mod transfer;

use build::BuildOptions;
use clap::{Parser, Subcommand};
use config::Config;
use std::{path::PathBuf, process};

#[derive(Parser)]
#[command(name = "deploy")]
#[command(version)]
#[command(about = "DeployTool - Remote deployment manager")]
struct Cli {
    #[arg(long, global = true, default_value = "deploy.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List configured targets
    List,

    /// Show target information
    Info { name: String },

    /// Preview full deployment
    Plan { name: String },

    /// Build Rust project
    Build {
        name: String,

        #[arg(long, conflicts_with = "release")]
        debug: bool,

        #[arg(long)]
        release: bool,

        #[arg(long)]
        clean: bool,

        #[arg(long)]
        verbose: bool,

        #[arg(long)]
        dry_run: bool,
    },

    /// Test SSH connection
    Test { name: String },

    /// Show remote service status
    Status { name: String },

    /// Display remote journal logs
    Logs {
        name: String,

        #[arg(long, default_value_t = 50)]
        lines: u32,

        #[arg(short, long)]
        follow: bool,
    },

    /// Transfer compiled binary to staging area
    Upload {
        name: String,

        #[arg(long)]
        dry_run: bool,
    },
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let config = Config::load(&cli.config)?;

    match cli.command {
        Commands::List => config.list(),

        Commands::Info { name } => {
            config.get_target(&name)?.print_info(&name);
        }

        Commands::Plan { name } => {
            config.get_target(&name)?.print_plan(&name);
        }

        Commands::Build {
            name,
            debug,
            release: _,
            clean,
            verbose,
            dry_run,
        } => {
            let options = BuildOptions {
                release: !debug,
                clean,
                verbose,
                dry_run,
            };

            build::build(&name, config.get_target(&name)?, options)?;
        }

        Commands::Test { name } => {
            ssh::test(config.get_target(&name)?)?;
        }

        Commands::Status { name } => {
            ssh::status(config.get_target(&name)?)?;
        }

        Commands::Logs {
            name,
            lines,
            follow,
        } => {
            ssh::logs(config.get_target(&name)?, lines, follow)?;
        }

        Commands::Upload { name, dry_run } => {
            transfer::upload(&name, config.get_target(&name)?, dry_run)?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        output::error(&error);
        process::exit(1);
    }
}
