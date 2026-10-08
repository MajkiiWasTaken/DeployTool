#!/usr/bin/env bash
# /************************************************
# * File: install.sh
# * Author: Michal Švrček
# *
# * DeployTool Linux installer
# *
# * ver. 0.5.0
# *************************************************/

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

SOURCE="${1:-${PROJECT_ROOT}/target/release/deploy}"
INSTALL_DIR="${HOME}/.local/bin"
DESTINATION="${INSTALL_DIR}/deploy"

echo "DeployTool Linux Installer"

if [[ ! -f "$SOURCE" ]]; then
    echo "ERROR: deploy binary not found: $SOURCE" >&2
    exit 1
fi

mkdir -p "$INSTALL_DIR"

SOURCE_ABS="$(realpath "$SOURCE")"

if [[ "$SOURCE_ABS" != "$DESTINATION" ]]; then
    install -m 755 "$SOURCE_ABS" "$DESTINATION"
fi

mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/deploytool"

PROFILE="$HOME/.profile"
PATH_LINE='export PATH="$HOME/.local/bin:$PATH"'

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    if ! grep -Fqx "$PATH_LINE" "$PROFILE" 2>/dev/null; then
        printf '\n# User executables\n%s\n' "$PATH_LINE" >> "$PROFILE"
    fi
fi

echo
echo "[OK] DeployTool installed successfully."
echo "Binary: $DESTINATION"
"$DESTINATION" --version

echo
echo "Ensure ~/.local/bin is in PATH."
echo "Then run:"
echo "  deploy --version"
echo "  deploy list"
