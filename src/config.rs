/************************************************
* File: config.rs
* Author: Michal Švrček
*
* DeployTool configuration and target management
*
* ver. 0.2.0
*************************************************/

use crate::output;
use colored::Colorize;
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
    #[serde(default)]
    pub ssh: SshConfig,
    #[serde(default)]
    pub build: BuildConfig,
}

#[derive(Debug, Deserialize)]
pub struct SshConfig {
    #[serde(default = "default_ssh_port")]
    pub port: u16,

    #[serde(default)]
    pub identity_file: Option<String>,

    #[serde(default)]
    pub legacy_scp: bool,
}

fn default_ssh_port() -> u16 {
    22
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            port: default_ssh_port(),
            identity_file: None,
            legacy_scp: false,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildRunner {
    #[default]
    Native,
    Wsl,
}

#[derive(Debug, Default, Deserialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub runner: BuildRunner,

    #[serde(default)]
    pub wsl_project: Option<String>,

    #[serde(default)]
    pub linker: Option<String>,
}

impl BuildRunner {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Native => "Native",
            Self::Wsl => "WSL",
        }
    }
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
        output::banner();
        output::heading("Available deployment targets");

        if self.targets.is_empty() {
            output::warning("No deployment targets configured.");
            return;
        }

        for (name, target) in &self.targets {
            println!(
                "  {} {}",
                format!("{name:<16}").cyan().bold(),
                format!("{}@{}", target.user, target.host).white()
            );
        }

        println!();
        output::success(&format!("{} target(s) configured", self.targets.len()));
    }
}

impl Target {
    pub fn print_info(&self, name: &str) {
        output::banner();
        output::heading(&format!("Target: {name}"));

        output::label("Host:", &self.host);
        output::label("User:", &self.user);
        output::label("Project:", &self.project);
        output::label("Architecture:", &self.architecture);
        output::label("Binary:", &self.binary);
        output::label("Service:", &self.service);
        output::label("Build runner:", self.build.runner.as_str());

        output::label("SSH port:", &self.ssh.port.to_string());
        output::label(
            "SCP protocol:",
            if self.ssh.legacy_scp {
                "Legacy SCP"
            } else {
                "SFTP"
            },
        );

        if let Some(path) = &self.build.wsl_project {
            output::label("WSL project:", path);
        }

        if let Some(linker) = &self.build.linker {
            output::label("Linker:", linker);
        }
    }

    pub fn print_plan(&self, name: &str) {
        output::banner();
        output::heading(&format!("Deployment plan: {name}"));

        output::step("1. Build project");
        output::label("Project:", &self.project);
        output::label("Runner:", self.build.runner.as_str());
        output::label("Architecture:", &self.architecture);

        output::step("2. Validate binary");
        output::label("Binary:", &self.binary);

        output::step("3. Connect via SSH");
        output::label("Destination:", &format!("{}@{}", self.user, self.host));

        output::step("4. Backup installed application");
        output::label("Service:", &self.service);

        output::step("5. Upload and install new binary");

        output::step("6. Restart service");
        output::label("Service:", &self.service);

        output::step("7. Verify service health");

        println!();
        output::warning("Preview only. No deployment actions were executed.");
    }
}
