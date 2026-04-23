#!/usr/bin/env bash
set -euo pipefail

echo "[post-create] Preparing in-container Rust environment..."
cargo fetch
cargo install --path . --locked --force
