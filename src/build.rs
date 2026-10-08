/************************************************
* File: build.rs
* Author: Michal Švrček
*
* DeployTool Cargo build and cross-compilation system
*
* ver. 0.2.0
*************************************************/

use crate::{
    config::{BuildRunner, Target},
    output,
};
use std::{
    path::Path,
    process::{Command, Stdio},
};

pub struct BuildOptions {
    pub release: bool,
    pub clean: bool,
    pub verbose: bool,
    pub dry_run: bool,
}

fn linker_variable(target: &str) -> String {
    let normalized = target
        .replace('-', "_")
        .replace('.', "_")
        .to_ascii_uppercase();

    format!("CARGO_TARGET_{normalized}_LINKER")
}

fn wsl_directory(target: &Target) -> Result<&str, String> {
    let path = target
        .build
        .wsl_project
        .as_deref()
        .ok_or("WSL build requires build.wsl_project")?;

    if !path.starts_with('/') {
        return Err("WSL project path must be an absolute Linux path".into());
    }

    Ok(path)
}

fn create_command(target: &Target, cargo_args: &[&str]) -> Result<Command, String> {
    let linker = target.build.linker.as_deref().filter(|s| !s.is_empty());

    match target.build.runner {
        BuildRunner::Native => {
            let project = Path::new(&target.project);

            if !project.join("Cargo.toml").is_file() {
                return Err(format!("Cargo.toml not found in {}", project.display()));
            }

            let mut command = Command::new("cargo");
            command.current_dir(project);
            command.args(cargo_args);

            if let Some(linker) = linker {
                command.env(linker_variable(&target.architecture), linker);
            }

            Ok(command)
        }

        BuildRunner::Wsl => {
            let path = wsl_directory(target)?;

            if !cfg!(windows) {
                return Err("WSL runner is only available on Windows".into());
            }

            let mut command = Command::new("wsl.exe");

            command.args(["--cd", path, "--"]);

            if let Some(linker) = linker {
                command.arg("env");
                command.arg(format!(
                    "{}={linker}",
                    linker_variable(&target.architecture)
                ));
            }

            command.arg("cargo");
            command.args(cargo_args);

            Ok(command)
        }
    }
}

fn execute(target: &Target, args: &[&str], options: &BuildOptions) -> Result<(), String> {
    if options.dry_run {
        output::info(&format!("Would execute: cargo {}", args.join(" ")));
        return Ok(());
    }

    let mut command = create_command(target, args)?;

    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to start build: {e}"))?;

    if !status.success() {
        return Err(format!("Cargo exited with status: {status}"));
    }

    Ok(())
}

fn verify_binary(target: &Target, profile: &str) -> Result<(), String> {
    let suffix = if target.architecture.contains("windows") {
        ".exe"
    } else {
        ""
    };

    let relative = format!(
        "target/{}/{}/{}{}",
        target.architecture, profile, target.binary, suffix
    );

    match target.build.runner {
        BuildRunner::Native => {
            let binary = Path::new(&target.project).join(&relative);

            let metadata = std::fs::metadata(&binary)
                .map_err(|e| format!("Binary not found at {}: {e}", binary.display()))?;

            if !metadata.is_file() || metadata.len() == 0 {
                return Err("Build artifact is not a valid nonempty file".into());
            }

            output::success(&format!("Binary verified: {}", binary.display()));
        }

        BuildRunner::Wsl => {
            let path = wsl_directory(target)?;

            let status = Command::new("wsl.exe")
                .args(["--cd", path, "--", "test", "-s", &relative])
                .status()
                .map_err(|e| format!("WSL verification failed: {e}"))?;

            if !status.success() {
                return Err(format!("Build artifact missing or empty: {relative}"));
            }

            output::success(&format!("Binary verified: {relative}"));
        }
    }

    Ok(())
}

pub fn build(name: &str, target: &Target, options: BuildOptions) -> Result<(), String> {
    output::banner();
    output::heading(&format!("Building target: {name}"));

    output::label("Runner:", target.build.runner.as_str());
    output::label("Architecture:", &target.architecture);

    let profile = if options.release { "release" } else { "debug" };
    output::label("Profile:", profile);

    if options.dry_run {
        output::warning("DRY RUN: No commands will be executed");
    }

    if options.clean {
        output::step("Cleaning build artifacts");

        let args = ["clean", "--target", target.architecture.as_str()];

        execute(target, &args, &options)?;

        if !options.dry_run {
            output::success("Cargo clean completed");
        }
    }

    output::step("Starting Cargo build");

    let mut args = vec!["build", "--target", target.architecture.as_str()];

    if options.release {
        args.push("--release");
    }

    if options.verbose {
        args.push("--verbose");
    }

    execute(target, &args, &options)?;

    if options.dry_run {
        output::success("Build preview completed");
        return Ok(());
    }

    output::success("Cargo build successful");

    output::step("Verifying build artifact");
    verify_binary(target, profile)?;

    output::success("Build completed successfully");

    Ok(())
}
