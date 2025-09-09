#!/bin/bash
set -euo pipefail

# Function to check if a command exists
command_exists() {
	command -v "$1" >/dev/null 2>&1
}

# Save the original branch
ORIGINAL_BRANCH=$(git branch --show-current)
echo "[INFO] Original branch: $ORIGINAL_BRANCH"

# Cleanup function
cleanup() {
	echo "[INFO] Cleaning up..."
	# Return to original directories (if pushd was used)
	while popd > /dev/null 2>&1; do
		echo "[INFO] Returned to previous directory"
	done
	
	# Return to original branch
	if [[ -n "${ORIGINAL_BRANCH:-}" ]]; then
		echo "[INFO] Switching back to original branch: $ORIGINAL_BRANCH"
		git checkout "$ORIGINAL_BRANCH" > /dev/null 2>&1 || echo "[WARNING] Could not switch back to original branch"
	fi
}

# Set trap to run cleanup on exit
trap cleanup EXIT

# Go to the root directory of the project
pushd "$(dirname "$0")/../.." > /dev/null
echo "[INFO] Current directory: $(pwd)"

echo "[INFO] Switching to 'tusk' branch..."
git checkout tusk

echo "[INFO] Pulling latest changes from remote..."
git pull origin tusk

echo "[INFO] Running install.sh..."
if [[ ! -f ./install.sh ]]; then
	echo "[ERROR] install.sh not found!" >&2
	exit 1
fi
bash ./install.sh

echo "[INFO] Compiling the project with cargo..."
if ! command_exists cargo; then
	echo "[ERROR] cargo not found!" >&2
	exit 1
fi
cargo build

echo "[INFO] Installing Python dependencies..."
pushd benchmark > /dev/null
if [[ ! -f requirements.txt ]]; then
	echo "[ERROR] requirements.txt not found!" >&2
	exit 1
fi
if ! command_exists python3; then
	echo "[ERROR] python3 not found!" >&2
	exit 1
fi
python3 -m pip install --upgrade pip
python3 -m pip install -r requirements.txt

echo "[INFO] Running benchmark for Claim 2..."
if ! command_exists fab; then
	echo "[ERROR] fab (Fabric) not found!" >&2
	exit 1
fi
fab artiworker

echo "[INFO] Script completed successfully."