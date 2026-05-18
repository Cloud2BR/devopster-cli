#!/usr/bin/env bash
set -euo pipefail

IMAGE_DEVCONTAINER="devopster-cli-devcontainer"
IMAGE_DEV="devopster-cli-dev"
WORKDIR="/workspaces/devopster-cli"

usage() {
  cat <<'EOF'
Usage: scripts/container.sh <task>

Container-only tasks (no local Rust install required):
  devcontainer-build   Build shared devcontainer image
  dev-build            Build runtime dev image with devopster installed
  shell                Open an interactive shell inside dev image
  bootstrap            cargo fetch + install + test inside devcontainer image
  build                cargo build --locked inside devcontainer image
  test                 cargo test --locked inside devcontainer image
  lint                 cargo clippy --all-targets --all-features -- -D warnings
  verify               Run build + test + lint inside devcontainer image
  setup                Run devopster setup in dev image
  run [args...]        Run devopster <args...> in dev image

Examples:
  scripts/container.sh devcontainer-build
  scripts/container.sh verify
  scripts/container.sh run repo audit
EOF
}

ensure_docker() {
  if ! command -v docker >/dev/null 2>&1; then
    echo "Docker is required but not installed or not on PATH." >&2
    exit 1
  fi
  if ! docker info >/dev/null 2>&1; then
    echo "Docker daemon is not reachable. Start Docker and retry." >&2
    exit 1
  fi
}

build_devcontainer() {
  docker build --target devcontainer -t "$IMAGE_DEVCONTAINER" .
}

build_dev() {
  docker build --target dev -t "$IMAGE_DEV" .
}

run_devcontainer_cmd() {
  docker run --rm \
    -v "$PWD:$WORKDIR" \
    -w "$WORKDIR" \
    "$IMAGE_DEVCONTAINER" \
    bash -lc "$1"
}

run_dev_cmd() {
  args=(run --rm)
  if [[ -t 0 && -t 1 ]]; then
    args+=( -it )
  fi
  args+=(
    -v "$HOME/.config/devopster:/root/.config/devopster"
    -v "$PWD:$WORKDIR"
    -w "$WORKDIR"
    "$IMAGE_DEV"
    bash -lc "$1"
  )
  docker "${args[@]}"
}

main() {
  task="${1:-}"
  shift || true

  if [[ -z "$task" ]]; then
    usage
    exit 1
  fi

  ensure_docker

  case "$task" in
    devcontainer-build)
      build_devcontainer
      ;;
    dev-build)
      build_dev
      ;;
    shell)
      build_dev
      run_dev_cmd "bash"
      ;;
    bootstrap)
      build_devcontainer
      run_devcontainer_cmd "cargo fetch && cargo install --path . --locked --force && cargo test --locked"
      ;;
    build)
      build_devcontainer
      run_devcontainer_cmd "cargo build --locked"
      ;;
    test)
      build_devcontainer
      run_devcontainer_cmd "cargo test --locked"
      ;;
    lint)
      build_devcontainer
      run_devcontainer_cmd "cargo clippy --all-targets --all-features -- -D warnings"
      ;;
    verify)
      build_devcontainer
      run_devcontainer_cmd "cargo build --locked && cargo test --locked && cargo clippy --all-targets --all-features -- -D warnings"
      ;;
    setup)
      build_dev
      run_dev_cmd "devopster setup"
      ;;
    run)
      if [[ $# -eq 0 ]]; then
        echo "Missing args for 'run'. Example: scripts/container.sh run repo list" >&2
        exit 1
      fi
      build_dev
      run_dev_cmd "devopster $*"
      ;;
    *)
      echo "Unknown task: $task" >&2
      usage
      exit 1
      ;;
  esac
}

main "$@"
