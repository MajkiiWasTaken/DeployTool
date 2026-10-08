/************************************************
* File: transfer.rs
* Author: Michal Švrček
*
* DeployTool staged binary transfer over SCP
*
* ver. 0.3.0
*************************************************/

use crate::{
    config::{BuildRunner, Target},
    output, ssh,
};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && !value.starts_with('-')
}

fn binary_path(target: &Target) -> Result<std::path::PathBuf, String> {
    if !safe_name(&target.binary) {
        return Err("Invalid binary name".into());
    }

    let relative = format!("target/{}/release/{}", target.architecture, target.binary);

    match target.build.runner {
        BuildRunner::Native | BuildRunner::Wsl => {
            let project = Path::new(&target.project);
            Ok(project.join(relative))
        }
    }
}

pub fn upload(name: &str, target: &Target, dry_run: bool) -> Result<(), String> {
    if !safe_name(name) {
        return Err("Invalid target name".into());
    }

    let source = binary_path(target)?;
    let remote_path = format!("/tmp/deploytool-{name}-{}.staged", target.binary);

    output::banner();
    output::heading(&format!("Staged upload: {name}"));

    output::label("Source:", &source.display().to_string());
    output::label("Destination:", &remote_path);

    if dry_run {
        output::warning("DRY RUN: Nothing will be uploaded");
        return Ok(());
    }

    let metadata = fs::metadata(&source)
        .map_err(|e| format!("Build artifact not found: {} ({e})", source.display()))?;

    if !metadata.is_file() || metadata.len() == 0 {
        return Err("Build artifact is empty or invalid".into());
    }

    let destination = ssh::destination(target)?;

    let mut command = Command::new("scp");

    command.args([
        "-o",
        "BatchMode=yes",
        "-o",
        "StrictHostKeyChecking=yes",
        "-o",
        "ConnectTimeout=10",
        "-P",
        &target.ssh.port.to_string(),
    ]);

    if target.ssh.legacy_scp {
        command.arg("-O");
    }

    if let Some(identity) = &target.ssh.identity_file {
        command.arg("-i").arg(identity);
    }

    command
        .arg(&source)
        .arg(format!("{destination}:{remote_path}"))
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    output::step("Transferring binary to staging location");

    let status = command
        .status()
        .map_err(|e| format!("Cannot start SCP: {e}"))?;

    if !status.success() {
        return Err(format!("SCP transfer failed: {status}"));
    }

    output::success("Binary transferred to staging location");
    output::warning("Firmware has NOT been installed or activated");

    Ok(())
}
