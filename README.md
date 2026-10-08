# DeployTool

**DeployTool** is a cross-platform command-line utility written in Rust for building, transferring, deploying, and managing applications on remote Linux devices.

It supports Cargo builds, custom Bash deployment scripts, SSH diagnostics, deployment history, and colored terminal output.

## Features

- Rust and Cargo build automation
- Native and Windows Subsystem for Linux (WSL) build runners
- Custom build and deployment scripts
- SSH connectivity testing
- Remote systemd service status and logs
- SCP binary staging
- Local deployment preflight checks
- Deployment history
- Per-target local deployment locking
- Dry-run support
- TOML-based configuration
- Windows and Linux support

## Installation

Download the latest package from [GitHub Releases](https://github.com/MajkiiWasTaken/DeployTool/releases).

### Windows

Extract `DeployTool-windows-x86_64.zip` and run:

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1 -BinaryPath .\deploy.exe
```

Restart your terminal and verify:

```powershell
deploy --version
```

### Linux

Extract the Linux package:

```bash
mkdir deploytool
tar -xzf DeployTool-linux-x86_64.tar.gz -C deploytool
cd deploytool
bash ./install.sh ./deploy
```

Ensure `~/.local/bin` is included in your PATH.

### Build from source

```bash
git clone https://github.com/MajkiiWasTaken/DeployTool.git
cd DeployTool
cargo build --release
```

Install using `scripts/install.ps1` on Windows or `scripts/install.sh` on Linux.

## Usage

| Command | Description |
|---|---|
| `deploy list` | List configured targets |
| `deploy info TestDevice` | Show target information |
| `deploy plan TestDevice` | Preview deployment configuration |
| `deploy build TestDevice` | Build the application |
| `deploy build TestDevice --dry-run` | Preview build commands |
| `deploy test TestDevice` | Test SSH connectivity |
| `deploy status TestDevice` | Show remote service status |
| `deploy logs TestDevice` | Read remote journal logs |
| `deploy logs TestDevice --follow` | Follow service logs |
| `deploy upload TestDevice --dry-run` | Preview staged upload |
| `deploy preflight TestDevice` | Run local prerequisite checks |
| `deploy TestDevice --dry-run` | Preview the configured deployment script |
| `deploy TestDevice --yes` | Execute the deployment script |
| `deploy history TestDevice` | View deployment history |
| `deploy history TestDevice --last` | View the latest deployment record |

**Warning:** `deploy TestDevice --yes` executes the configured deployment script, which may modify remote services, files, firmware, and network configuration.

## Configuration

DeployTool uses TOML configuration files.

Example configuration:

```toml
[targets.TestDevice]
host = "192.0.2.10"
user = "admin"
project = 'C:\Projects\TestProject'
architecture = "aarch64-unknown-linux-gnu"
binary = "test_app"
service = "test-app.service"

[targets.TestDevice.build]
runner = "wsl"
method = "script"
script = "scripts/build.sh"
wsl_project = "/mnt/c/Projects/TestProject"

[targets.TestDevice.deployment]
method = "script"
script = "scripts/deploy.sh"

[targets.TestDevice.ssh]
port = 22
legacy_scp = false
```

The addresses and paths above are examples and must be replaced with actual values.

### Configuration locations

On Windows:

`%APPDATA%\DeployTool\deploy.toml`

On Linux:

`~/.config/deploytool/deploy.toml`

An explicit `--config` path takes precedence. Otherwise, DeployTool first checks the current working directory, then the user configuration directory.

To inspect a particular configuration:

```bash
deploy --config ./deploy.toml info TestDevice
```

## Build backends

### Cargo

The Cargo backend executes a standard Rust build targeting the configured architecture.

### Script

The script backend executes a custom Bash build script using the configured native or WSL runner.

This allows projects to retain existing build systems, cross-compilation requirements, signing procedures, and validation steps.

## Deployment

DeployTool can invoke an existing deployment script.

The deployment script remains responsible for the actual remote update process, including any backup, artifact validation, service restart, and rollback.

DeployTool does not automatically guarantee successful recovery from every failure.

For production deployments, use scripts that verify software authenticity, product identity, and artifact integrity.

## Deployment history

Deployment records include execution timestamps, duration, and success or failure status.

History locations:

- Windows: `%APPDATA%\DeployTool\history`
- Linux: `~/.local/state/deploytool/history`

Records track script execution results. They are not a substitute for a device-side firmware audit log.

## Development

```bash
cargo fmt
cargo check
cargo test
cargo build --release
```

## Roadmap

- Configurable cross-compilation environments
- Extended deployment preflight checks
- Remote deployment locking
- Build artifact integrity verification
- Structured deployment logs
- Multiple host profiles and SSH configuration improvements
- Advanced rollback orchestration

## Author

**Michal Švrček**

## License

Distributed under the MIT License. See [LICENSE](LICENSE).
