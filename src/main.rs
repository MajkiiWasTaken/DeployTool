/************************************************
* File: main.rs
* Author: Michal Švrček
*
* DeployTool CLI entry point and command handling
*
* ver. 0.3.1
*************************************************/

mod build;
mod config;
mod deployment;
mod history;
mod output;
mod preflight;
mod script_runner;
mod ssh;
mod transfer;

use build::BuildOptions;
use clap::{Parser, Subcommand};
use config::{BuildMethod, Config, Target};
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
    /// List configured deployment targets
    List,

    /// Show target information
    Info { name: String },

    /// Preview the deployment configuration
    Plan { name: String },

    /// Build a configured project
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

    /// Execute configured deployment script
    Run {
        name: String,

        /// Confirm execution of remote deployment
        #[arg(long, conflicts_with = "dry_run")]
        yes: bool,

        /// Preview script without executing it
        #[arg(long)]
        dry_run: bool,
    },

    /// Test SSH connectivity
    Test { name: String },

    /// Show remote systemd status
    Status { name: String },

    /// Display remote journal logs
    Logs {
        name: String,

        #[arg(long, default_value_t = 50)]
        lines: u32,

        #[arg(short, long)]
        follow: bool,
    },

    /// Upload binary to remote staging area
    Upload {
        name: String,

        #[arg(long)]
        dry_run: bool,
    },

    /// Check local deployment prerequisites
    Preflight { name: String },

    /// Show deployment history
    History {
        name: String,

        /// Show only the latest record
        #[arg(long)]
        last: bool,
    },

    /// Deploy using the target name directly
    #[command(external_subcommand)]
    Target(Vec<String>),
}

fn build_target(name: &str, target: &Target, options: BuildOptions) -> Result<(), String> {
    match target.build.method {
        BuildMethod::Cargo => build::build(name, target, options),

        BuildMethod::Script => {
            if !options.release || options.clean || options.verbose {
                return Err("Script builds do not support --debug, --clean or --verbose".into());
            }

            let script = target
                .build
                .script
                .as_deref()
                .ok_or("Build script not configured")?;

            script_runner::execute(name, target, script, "Build", options.dry_run)
        }
    }
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

            build_target(&name, config.get_target(&name)?, options)?;
        }

        Commands::Run { name, yes, dry_run } => {
            deployment::execute(&name, config.get_target(&name)?, yes, dry_run)?;
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

        Commands::Preflight { name } => {
            preflight::check(&name, config.get_target(&name)?)?;
        }

        Commands::History { name, last } => {
            config.get_target(&name)?;
            history::display(&name, last)?;
        }

        Commands::Target(args) => {
            let Some(name) = args.first() else {
                return Err("Usage: deploy <target> [--dry-run | --yes]".into());
            };

            let mut dry_run = false;
            let mut yes = false;

            for arg in args.iter().skip(1) {
                match arg.as_str() {
                    "--dry-run" => dry_run = true,
                    "--yes" => yes = true,
                    _ => return Err(format!("Unknown argument: {arg}")),
                }
            }

            if dry_run && yes {
                return Err("--dry-run and --yes cannot be used together".into());
            }

            deployment::execute(name, config.get_target(name)?, yes, dry_run)?;
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
