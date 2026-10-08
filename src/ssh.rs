/************************************************
* File: ssh.rs
* Author: Michal Švrček
*
* DeployTool SSH connectivity and remote commands
*
* ver. 0.3.0
*************************************************/

use crate::{config::Target, output};
use std::process::{Command, Stdio};

fn safe_part(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
        && !value.starts_with('-')
}

pub fn destination(target: &Target) -> Result<String, String> {
    if !safe_part(&target.user) || !safe_part(&target.host) {
        return Err("Invalid SSH username or hostname".into());
    }

    if target.ssh.port == 0 {
        return Err("Invalid SSH port".into());
    }

    Ok(format!("{}@{}", target.user, target.host))
}

pub fn base_command(target: &Target) -> Result<Command, String> {
    destination(target)?;

    let mut command = Command::new("ssh");

    command.args([
        "-o",
        "BatchMode=yes",
        "-o",
        "StrictHostKeyChecking=yes",
        "-o",
        "ConnectTimeout=10",
        "-p",
        &target.ssh.port.to_string(),
    ]);

    if let Some(identity) = &target.ssh.identity_file {
        command.arg("-i").arg(identity);
    }

    Ok(command)
}

pub fn run(target: &Target, remote_command: &str, follow: bool) -> Result<(), String> {
    let mut command = base_command(target)?;

    command
        .arg(destination(target)?)
        .arg(remote_command)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if follow {
        output::info("Press Ctrl+C to stop following logs");
    }

    let status = command
        .status()
        .map_err(|e| format!("Cannot start SSH: {e}"))?;

    if !status.success() {
        return Err(format!("SSH command failed: {status}"));
    }

    Ok(())
}

pub fn test(target: &Target) -> Result<(), String> {
    output::banner();
    output::heading("SSH connectivity test");

    output::label("Destination:", &destination(target)?);
    output::label("Port:", &target.ssh.port.to_string());

    output::step("Connecting to remote device");

    run(target, "printf 'SSH connection established\\n'", false)?;

    output::success("SSH connection successful");
    Ok(())
}

fn validated_service(target: &Target) -> Result<&str, String> {
    if !safe_part(&target.service) {
        return Err("Invalid systemd service name".into());
    }

    Ok(&target.service)
}

pub fn status(target: &Target) -> Result<(), String> {
    output::banner();
    output::heading("Remote service status");

    let service = validated_service(target)?;
    output::label("Service:", service);

    run(
        target,
        &format!("systemctl status --no-pager --full {service}"),
        false,
    )
}

pub fn logs(target: &Target, lines: u32, follow: bool) -> Result<(), String> {
    if lines == 0 || lines > 10_000 {
        return Err("Log line count must be between 1 and 10000".into());
    }

    let service = validated_service(target)?;

    output::banner();
    output::heading("Remote service logs");
    output::label("Service:", service);

    let mut command = format!("journalctl -u {service} -n {lines} --no-pager");

    if follow {
        command.push_str(" -f");
    }

    run(target, &command, follow)
}
