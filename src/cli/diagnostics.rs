use std::process::Command;

use anyhow::Result;
use clap::Args;

use crate::cli::container_runtime::ensure_docker_ready;
use crate::ui;

#[derive(Debug, Args)]
pub struct DiagnosticsCommand {}

impl DiagnosticsCommand {
    pub async fn run(&self) -> Result<()> {
        ui::header("devopster diagnostics");

        check_tool("docker", true)?;
        check_tool("gh", false)?;
        check_tool("az", false)?;
        check_tool("glab", false)?;

        ui::section("Docker daemon");
        ensure_docker_ready()?;
        ui::success("Docker daemon is reachable.");

        ui::section("Summary");
        ui::success("Diagnostics complete.");
        Ok(())
    }
}

fn check_tool(name: &str, required: bool) -> Result<()> {
    let status = Command::new(name).arg("--version").status();
    let available = status.map(|s| s.success()).unwrap_or(false);

    ui::section(&format!("Tool: {name}"));
    if available {
        ui::success("installed");
        return Ok(());
    }

    if required {
        anyhow::bail!("required tool '{}' is not installed or not on PATH", name);
    }

    ui::warn("not installed (optional)");
    Ok(())
}
