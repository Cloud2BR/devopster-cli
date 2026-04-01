#!/usr/bin/env bash
set -euo pipefail

echo "==> devopster setup"

# ── Install Docker if missing ─────────────────────────────────────────────────
if command -v docker > /dev/null 2>&1; then
    echo "==> Docker already installed: $(docker --version)"
else
    if [[ "$(uname -s)" == "Darwin" ]]; then
        echo "==> Installing Docker Desktop via Homebrew..."
        if ! command -v brew > /dev/null 2>&1; then
            echo "==> Installing Homebrew first..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        brew install --cask docker
        echo ""
        echo "==> Docker Desktop installed."
        echo "    Please start Docker Desktop from Applications, then re-run: make setup"
        exit 0
    elif [[ -f /etc/debian_version ]]; then
        echo "==> Installing Docker Engine via apt..."
        sudo apt-get update -qq
        sudo apt-get install -y ca-certificates curl gnupg lsb-release
        sudo install -m 0755 -d /etc/apt/keyrings
        curl -fsSL https://download.docker.com/linux/debian/gpg \
            | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
https://download.docker.com/linux/debian $(lsb_release -cs) stable" \
            | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
        sudo apt-get update -qq
        sudo apt-get install -y docker-ce docker-ce-cli containerd.io \
            docker-buildx-plugin docker-compose-plugin
        sudo usermod -aG docker "$USER"
        echo "==> Docker installed. You may need to log out and back in for group changes."
    else
        echo "==> Unsupported OS. Install Docker manually: https://docs.docker.com/get-docker/"
        exit 1
    fi
fi

# ── Build image ──────────────────────────────────────────────────────────────
echo "==> Building devopster container image..."
make container-build

# ── Host browser proxy ───────────────────────────────────────────────────────
# devopster writes a URL to .devopster_open_url (inside the mounted /app dir)
# when it needs to open the browser from inside the container.  We watch that
# file here on the host and call open/xdg-open so the browser pops up on the
# host machine automatically — no manual URL copying needed.
OPEN_FILE="$(pwd)/.devopster_open_url"
: > "$OPEN_FILE"
_last_url=""
(
    while true; do
        _url=$(cat "$OPEN_FILE" 2>/dev/null | tr -d '[:space:]')
        if [[ -n "$_url" && "$_url" != "$_last_url" ]]; then
            _last_url="$_url"
            if [[ "$(uname -s)" == "Darwin" ]]; then
                open "$_url"
            else
                xdg-open "$_url" 2>/dev/null || true
            fi
        fi
        sleep 0.3
    done
) &
_BROWSER_PID=$!

# Clean up the watcher and temp file when the container session ends.
trap 'kill "$_BROWSER_PID" 2>/dev/null || true; rm -f "$OPEN_FILE"' EXIT INT TERM

echo ""
echo "==> Opening container shell — browser will open automatically when you run devopster login."
docker run --rm -it \
    -v "$HOME/.config/devopster:/root/.config/devopster" \
    -v "$(pwd):/app" \
    -w /app \
    devopster-cli-dev \
    bash
