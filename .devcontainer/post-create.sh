#!/usr/bin/env bash
set -euo pipefail

echo "[post-create] Running in-container bootstrap..."
cargo fetch
cargo install --path . --locked --force
cargo test
