/************************************************
* File: config.rs
* Author: Michal Švrček
*
* DeployTool configuration and target management
*
* ver. 0.3.1
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
    pub build: BuildConfig,

    #[serde(default)]
    pub ssh: SshConfig,

    #[serde(default)]
    pub deployment: Option<DeploymentConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildRunner {
    #[default]
    Native,
    Wsl,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildMethod {
    #[default]
    Cargo,
    Script,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentMethod {
    #[default]
    Script,
}

#[derive(Debug, Default, Deserialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub runner: BuildRunner,

    #[serde(default)]
    pub method: BuildMethod,

    #[serde(default)]
    pub script: Option<String>,

    #[serde(default)]
    pub wsl_project: Option<String>,

    #[serde(default)]
    pub linker: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeploymentConfig {
    #[serde(default)]
    pub method: DeploymentMethod,

    pub script: String,
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
            port: 22,
            identity_file: None,
            legacy_scp: false,
        }
    }
}

impl BuildRunner {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Native => "Native",
            Self::Wsl => "WSL",
        }
    }
}

impl BuildMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cargo => "Cargo",
            Self::Script => "Script",
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
            output::warning("No targets configured.");
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
        output::label("SSH port:", &self.ssh.port.to_string());

        output::label("Build runner:", self.build.runner.as_str());
        output::label("Build method:", self.build.method.as_str());

        if let Some(path) = &self.build.wsl_project {
            output::label("WSL project:", path);
        }

        if let Some(script) = &self.build.script {
            output::label("Build script:", script);
        }

        if let Some(linker) = &self.build.linker {
            output::label("Linker:", linker);
        }

        if let Some(deployment) = &self.deployment {
            output::label("Deploy script:", &deployment.script);
        }
    }

    pub fn print_plan(&self, name: &str) {
        output::banner();
        output::heading(&format!("Deployment plan: {name}"));

        output::step("Build");
        output::label("Runner:", self.build.runner.as_str());
        output::label("Method:", self.build.method.as_str());
        output::label("Architecture:", &self.architecture);

        if let Some(script) = &self.build.script {
            output::label("Script:", script);
        }

        output::step("Deployment");

        match &self.deployment {
            Some(deployment) => {
                output::label("Method:", "Script");
                output::label("Script:", &deployment.script);
                output::label("Destination:", &format!("{}@{}", self.user, self.host));
            }
            None => {
                output::warning("No deployment backend configured.");
            }
        }

        println!();
        output::warning("Plan only. No commands were executed.");
    }
}
