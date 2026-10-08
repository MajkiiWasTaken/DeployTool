/************************************************
* File: preflight.rs
* Author: Michal Švrček
*
* DeployTool local deployment preflight checks
*
* ver. 0.4.0
*************************************************/

use crate::{
    config::{BuildRunner, Target},
    output,
};
use std::{
    path::Path,
    process::{Command, Stdio},
};

fn check_command(mut command: Command, description: &str) -> Result<(), String> {
    let status = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("{description}: {e}"))?;

    if status.success() {
        output::success(description);
        Ok(())
    } else {
        Err(format!("{description} failed: {status}"))
    }
}

pub fn check(name: &str, target: &Target) -> Result<(), String> {
    output::banner();
    output::heading(&format!("Preflight: {name}"));

    let deployment = target
        .deployment
        .as_ref()
        .ok_or("No deployment backend configured")?;

    let script = &deployment.script;

    // Reject parent traversal and absolute script paths.
    crate::script_runner::validate_script(script)?;

    match target.build.runner {
        BuildRunner::Wsl => {
            if !cfg!(windows) {
                return Err("WSL runner requires Windows".into());
            }

            let project = target
                .build
                .wsl_project
                .as_deref()
                .ok_or("WSL project path not configured")?;

            if !project.starts_with('/') {
                return Err("WSL project must be an absolute path".into());
            }

            let mut directory_check = Command::new("wsl.exe");
            directory_check.args(["--cd", project, "--", "/usr/bin/test", "-d", "."]);

            check_command(directory_check, "WSL project directory accessible")?;

            let mut script_check = Command::new("wsl.exe");
            script_check.args(["--cd", project, "--", "/usr/bin/test", "-f", script]);

            check_command(script_check, "Deployment script exists")?;

            // Reuse the configured script environment, which already
            // supplies the Cargo and Zig search paths.
            // Tool resolution is delegated to the script itself.
            output::info(
                "Build tools and signing requirements are validated by the Gateway script.",
            );
        }

        BuildRunner::Native => {
            let project = Path::new(&target.project);

            if !project.is_dir() {
                return Err(format!("Project directory missing: {}", project.display()));
            }

            let script_path = project.join(script);

            if !script_path.is_file() {
                return Err(format!(
                    "Deployment script missing: {}",
                    script_path.display()
                ));
            }

            output::success("Project directory accessible");
            output::success("Deployment script exists");

            let mut bash = Command::new("bash");
            bash.arg("--version");

            check_command(bash, "Bash available")?;
        }
    }

    output::success("Local preflight completed");
    output::warning("This is not a remote health check or firmware signature verification.");

    Ok(())
}
