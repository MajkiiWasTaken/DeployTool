/************************************************
* File: config.rs
* Author: Michal Švrček
*
* DeployTool configuration management
*
* ver. 0.1.0
*************************************************/

use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub targets: BTreeMap<String, Target>,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub host: String,
    pub user: String,
    pub project: String,
    pub architecture: String,
    pub binary: String,
    pub service: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;

        toml::from_str(&content).map_err(|e| format!("Invalid configuration: {e}"))
    }

    pub fn get_target(&self, name: &str) -> Result<&Target, String> {
        self.targets
            .get(name)
            .ok_or_else(|| format!("Unknown target: {name}"))
    }

    pub fn list(&self) {
        if self.targets.is_empty() {
            println!("No deployment targets configured.");
            return;
        }

        println!("Available deployment targets:\n");

        for (name, target) in &self.targets {
            println!("  {:<16} {}@{}", name, target.user, target.host);
        }
    }
}

impl Target {
    pub fn print_info(&self, name: &str) {
        println!("DeployTool target: {name}");
        println!("----------------------------------------");
        println!("Host:         {}", self.host);
        println!("User:         {}", self.user);
        println!("Project:      {}", self.project);
        println!("Architecture: {}", self.architecture);
        println!("Binary:       {}", self.binary);
        println!("Service:      {}", self.service);
    }

    pub fn print_plan(&self, name: &str) {
        println!("[DRY RUN] Deployment plan: {name}\n");

        println!("1. Build project");
        println!("   Project: {}", self.project);
        println!("   Target:  {}", self.architecture);

        println!("\n2. Validate binary");
        println!("   Binary: {}", self.binary);

        println!("\n3. Connect via SSH");
        println!("   Host: {}@{}", self.user, self.host);

        println!("\n4. Backup installed application");
        println!("   Service: {}", self.service);

        println!("\n5. Upload and install new binary");

        println!("\n6. Restart service");
        println!("   Service: {}", self.service);

        println!("\n7. Verify service health");
        println!("   Roll back if verification fails");

        println!("\nNo deployment actions were executed.");
    }
}
