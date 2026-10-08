/************************************************
* File: script_runner.rs
* Author: Michal Švrček
*
* DeployTool custom Bash build and deployment runner
*
* ver. 0.3.1
*************************************************/

use crate::{
    config::{BuildRunner, Target},
    output,
};
use std::{
    path::{Component, Path},
    process::{Command, Stdio},
};

pub fn validate_script(script: &str) -> Result<(), String> {
    let path = Path::new(script);

    if path.is_absolute() || path.as_os_str().is_empty() {
        return Err("Script must be a relative path".into());
    }

    if !path
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err("Script path cannot contain parent directories".into());
    }

    if path.extension().is_none_or(|ext| ext != "sh") {
        return Err("Only .sh scripts are supported".into());
    }

    Ok(())
}

fn create_command(target: &Target, script: &str) -> Result<Command, String> {
    validate_script(script)?;

    match target.build.runner {
        BuildRunner::Wsl => {
            if !cfg!(windows) {
                return Err("WSL runner requires Windows".into());
            }

            let project = target
                .build
                .wsl_project
                .as_deref()
                .ok_or("WSL project directory is not configured")?;

            if !project.starts_with('/') {
                return Err("WSL project directory must be absolute".into());
            }

            let mut command = Command::new("wsl.exe");

            command
                .arg("--cd")
                .arg(project)
                .arg("--")
                .arg("/usr/bin/env")
                .arg("PATH=/home/svrcekm/.local/bin:/home/svrcekm/.cargo/bin:/usr/local/bin:/usr/bin:/bin")
                .arg("/usr/bin/bash")
                .arg(script);

            Ok(command)
        }

        BuildRunner::Native => {
            let project = Path::new(&target.project);

            if !project.is_dir() {
                return Err(format!(
                    "Project directory not found: {}",
                    project.display()
                ));
            }

            let mut command = Command::new("bash");
            command.arg(script).current_dir(project);

            Ok(command)
        }
    }
}

pub fn execute(
    name: &str,
    target: &Target,
    script: &str,
    action: &str,
    dry_run: bool,
) -> Result<(), String> {
    output::banner();
    output::heading(&format!("{action}: {name}"));

    output::label("Runner:", target.build.runner.as_str());
    output::label("Script:", script);

    if let Some(path) = &target.build.wsl_project {
        output::label("WSL directory:", path);
    }

    let mut command = create_command(target, script)?;

    if dry_run {
        output::warning("DRY RUN: Script will not be executed.");
        output::info(&format!("Prepared command: {command:?}"));
        return Ok(());
    }

    output::step(&format!("Executing {script}"));

    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Cannot start script: {e}"))?;

    if !status.success() {
        return Err(format!("{action} script failed with status: {status}"));
    }

    output::success(&format!("{action} completed successfully"));

    Ok(())
}
