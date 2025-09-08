
#!/bin/bash
set -euo pipefail

# Function to check if a command exists
command_exists() {
	command -v "$1" >/dev/null 2>&1
}

# Go to the root directory
pushd "$(dirname "$0")/../../.." > /dev/null

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

echo "[INFO] Running benchmark for Claim 1..."
if ! command_exists fab; then
	echo "[ERROR] fab (Fabric) not found!" >&2
	exit 1
fi
fab artifrontrunner
popd > /dev/null
popd > /dev/null