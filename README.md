# devopster-cli

Cross-platform GitOps CLI built in Rust for managing organization repositories across GitHub, Azure DevOps, and GitLab.

## What It Is

This project is a container-first CLI for repository governance and maintenance at scale. The goal is to give you a single tool that can scaffold repositories, audit standards, sync shared files, generate catalogs, and align metadata across multiple source-control platforms.

## Why This Repo Exists

Managing an organization with many repositories usually leads to repeated manual work:

- keeping repo metadata aligned
- copying workflows and templates between repos
- auditing descriptions, topics, and licensing
- scaffolding new repositories from a standard template
- generating a reusable catalog of projects

`devopster` is intended to centralize that work behind one CLI and one config file.

## Current Foundation

The repository now includes:

- a Rust CLI package with the `devopster` binary
- a command tree for `init`, `repo`, `catalog`, `topics`, and `stats`
- a provider abstraction for GitHub, Azure DevOps, and GitLab
- YAML configuration loading through `devopster-config.yaml`
- a dev container and Docker-based workflow so the host machine does not need Rust installed
- a CI workflow that builds and tests through containers

## Container-First Workflow

This project is designed to be developed inside a container.

### VS Code Dev Container

1. Clone the repository.
2. Open it in VS Code.
3. Reopen the folder in the Dev Container.
4. The post-create step runs `make setup` automatically.

### Local Commands

```bash
make setup
make build
make test
make run ARGS="stats"
make container-build
make container-test
```

## Example Commands

```bash
devopster init
devopster repo list
devopster repo audit
devopster repo scaffold --name sample-repo --template azure-overview
devopster catalog generate
devopster topics align
devopster stats
```

## Configuration

Start from the example file:

```bash
cp devopster-config.example.yaml devopster-config.yaml
```

Then set the provider and token environment variables you want to use.

## Next Steps

The current scaffold is intentionally focused on the architecture and containerized workflow. The next implementation steps are:

- wire real provider SDKs and REST clients
- add repository creation and file sync logic
- implement policy-based auditing
- render templates for new repository scaffolding
- generate machine-readable and web-friendly org catalogs