use anyhow::Result;
use clap::Args;

use crate::cli::container_runtime::{build_dev_image, ensure_docker_ready, run_in_dev_container};
use crate::ui;

#[derive(Debug, Args)]
pub struct DevEnvCommand {
    /// Docker image tag used for the local developer container
    #[arg(long, default_value = "devopster-cli-dev")]
    pub image: String,

    /// Skip rebuilding the container image before launch
    #[arg(long)]
    pub no_build: bool,

    /// Skip running `devopster setup` after bootstrap inside the container
    #[arg(long)]
    pub no_onboarding: bool,
}

impl DevEnvCommand {
    pub async fn run(&self) -> Result<()> {
        ui::header("devopster local developer environment");

        ui::section("Check Docker");
        ensure_docker_ready()?;
        ui::success("Docker is available and running.");

        if !self.no_build {
            ui::section("Build container image");
            build_dev_image(&self.image)?;
        }

        ui::section("Start local container");
        let in_container_cmd = if self.no_onboarding {
            "cargo fetch && cargo install --path . --locked --force && cargo test"
        } else {
            "cargo fetch && cargo install --path . --locked --force && cargo test && devopster setup"
        };

        run_in_dev_container(&self.image, in_container_cmd, true)?;
        ui::success("Local containerized developer environment completed.");

        Ok(())
    }
}
