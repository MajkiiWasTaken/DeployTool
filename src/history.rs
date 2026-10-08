/************************************************
* File: history.rs
* Author: Michal Švrček
*
* DeployTool deployment history and audit records
*
* ver. 0.4.0
*************************************************/

use crate::output;
use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub target: String,
    pub started: DateTime<Utc>,
    pub finished: DateTime<Utc>,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub duration_seconds: f64,
}

pub fn state_dir() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let appdata = env::var_os("APPDATA").ok_or("APPDATA is not available")?;

        Ok(PathBuf::from(appdata).join("DeployTool"))
    }

    #[cfg(not(windows))]
    {
        let base = env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
            .ok_or("Cannot locate user state directory")?;

        Ok(base.join("deploytool"))
    }
}

pub fn save(record: &DeploymentRecord) -> Result<(), String> {
    let directory = state_dir()?.join("history");

    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;

    let timestamp = record.started.format("%Y%m%dT%H%M%S%.fZ");
    let filename = format!("{}-{}.json", record.target, timestamp);

    let file = directory.join(filename);

    let json = serde_json::to_string_pretty(record).map_err(|e| e.to_string())?;

    fs::write(file, json).map_err(|e| e.to_string())
}

pub fn display(name: &str, last_only: bool) -> Result<(), String> {
    let directory = state_dir()?.join("history");

    output::banner();
    output::heading(&format!("Deployment history: {name}"));

    if !directory.exists() {
        output::warning("No deployment history available.");
        return Ok(());
    }

    let mut records = Vec::new();

    for entry in fs::read_dir(&directory).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();

        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }

        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;

        let record: DeploymentRecord =
            serde_json::from_str(&content).map_err(|e| format!("Invalid history file: {e}"))?;

        if record.target == name {
            records.push(record);
        }
    }

    records.sort_by(|a, b| b.started.cmp(&a.started));

    if records.is_empty() {
        output::warning("No deployment records for this target.");
        return Ok(());
    }

    let limit = if last_only { 1 } else { 20 };

    for record in records.iter().take(limit) {
        let date = record
            .started
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S");

        let status = if record.success {
            "SUCCESS".green().bold()
        } else {
            "FAILED".red().bold()
        };

        println!("  {}  {:<8}  {:.1}s", date, status, record.duration_seconds);
    }

    Ok(())
}
