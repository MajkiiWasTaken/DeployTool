/************************************************
* File: deployment.rs
* Author: Michal Švrček
*
* DeployTool deployment orchestration and locking
*
* ver. 0.4.0
*************************************************/

use crate::{
    config::Target,
    history::{self, DeploymentRecord},
    output, preflight, script_runner,
};
use chrono::Utc;
use fs2::FileExt;
use std::{
    fs::{self, OpenOptions},
    time::Instant,
};

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

pub fn execute(name: &str, target: &Target, yes: bool, dry_run: bool) -> Result<(), String> {
    if !valid_name(name) {
        return Err("Invalid deployment target name".into());
    }

    let deployment = target
        .deployment
        .as_ref()
        .ok_or("Deployment backend not configured")?;

    match deployment.method {
        crate::config::DeploymentMethod::Script => {}
    }

    if dry_run {
        return script_runner::execute(name, target, &deployment.script, "Deployment", true);
    }

    if !yes {
        return Err("Deployment requires --yes. Use --dry-run to preview.".into());
    }

    // Acquire the lock before preflight and hold it through
    // the entire deployment and history write.
    let lock_dir = history::state_dir()?.join("locks");

    fs::create_dir_all(&lock_dir).map_err(|e| format!("Cannot create lock directory: {e}"))?;

    let lock_path = lock_dir.join(format!("{name}.lock"));

    let lock_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| format!("Cannot open deployment lock: {e}"))?;

    lock_file
        .try_lock_exclusive()
        .map_err(|_| format!("Target '{name}' is already being deployed by another process."))?;

    preflight::check(name, target)?;

    output::banner();
    output::heading(&format!("Deploying: {name}"));
    output::warning("Remote device changes are now enabled.");

    let started = Utc::now();
    let timer = Instant::now();

    let result = script_runner::execute(name, target, &deployment.script, "Deployment", false);

    let duration = timer.elapsed();

    let record = DeploymentRecord {
        target: name.to_string(),
        started,
        finished: Utc::now(),
        success: result.is_ok(),
        exit_code: None,
        duration_seconds: duration.as_secs_f64(),
    };

    if let Err(error) = history::save(&record) {
        output::warning(&format!("Failed to save deployment history: {error}"));
    }

    // File lock is released when lock_file is dropped.
    drop(lock_file);

    output::banner();
    output::heading("Deployment summary");
    output::label("Target:", name);
    output::label(
        "Duration:",
        &format!("{:.1} seconds", duration.as_secs_f64()),
    );

    match result {
        Ok(()) => {
            output::success("Deployment script finished successfully");
            Ok(())
        }
        Err(error) => {
            output::error("Deployment failed or its script returned an error");
            output::warning("Check remote service status and rollback logs before retrying.");
            Err(error)
        }
    }
}
