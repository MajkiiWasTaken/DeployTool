#!/usr/bin/env bash
# /************************************************
# * File: uninstall.sh
# * Author: Michal Švrček
# *
# * DeployTool Linux uninstaller
# *
# * ver. 0.5.0
# *************************************************/

set -euo pipefail

DESTINATION="$HOME/.local/bin/deploy"

echo "DeployTool Linux Uninstaller"

if [[ -f "$DESTINATION" ]]; then
    rm -- "$DESTINATION"
fi

echo
echo "[OK] DeployTool uninstalled."
echo "Configuration and deployment history were preserved."
echo "PATH settings were left unchanged for other tools."
