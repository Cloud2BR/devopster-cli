use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cli::container_runtime::{build_dev_image, ensure_docker_ready, run_in_dev_container};
use crate::ui;

#[derive(Debug, Args)]
pub struct DevCommand {
    #[command(subcommand)]
    pub action: DevAction,

    /// Docker image tag used for containerized developer tasks
    #[arg(long, default_value = "devopster-cli-dev")]
    pub image: String,

    /// Skip rebuilding the container image before executing task(s)
    #[arg(long)]
    pub no_build: bool,
}

#[derive(Debug, Subcommand)]
pub enum DevAction {
    /// Fetch dependencies, install devopster, and run tests in container
    Bootstrap,
    /// Build devopster in container
    Build,
    /// Run tests in container
    Test,
    /// Run clippy in container
    Lint,
    /// Run build, test, and lint in container
    Verify,
}

impl DevCommand {
    pub async fn run(&self) -> Result<()> {
        ui::header("devopster developer automation");
        ui::section("Check Docker");
        ensure_docker_ready()?;
        ui::success("Docker is available and running.");

        if !self.no_build {
            ui::section("Build container image");
            build_dev_image(&self.image)?;
        }

        match self.action {
            DevAction::Bootstrap => {
                ui::section("Bootstrap");
                run_in_dev_container(
                    &self.image,
                    "cargo fetch && cargo install --path . --locked --force && cargo test",
                    false,
                )?;
            }
            DevAction::Build => {
                ui::section("Build");
                run_in_dev_container(&self.image, "cargo build", false)?;
            }
            DevAction::Test => {
                ui::section("Test");
                run_in_dev_container(&self.image, "cargo test", false)?;
            }
            DevAction::Lint => {
                ui::section("Lint");
                run_in_dev_container(
                    &self.image,
                    "cargo clippy --all-targets --all-features -- -D warnings",
                    false,
                )?;
            }
            DevAction::Verify => {
                ui::section("Verify");
                run_in_dev_container(
                    &self.image,
                    "cargo build && cargo test && cargo clippy --all-targets --all-features -- -D warnings",
                    false,
                )?;
            }
        }

        ui::success("Developer task completed.");
        Ok(())
    }
}
