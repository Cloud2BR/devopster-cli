use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;

#[derive(Debug, Args)]
pub struct InitCommand {
    #[arg(long, default_value = "devopster-config.yaml")]
    pub output: String,
}

impl InitCommand {
    pub async fn run(&self, config_path: &str) -> Result<()> {
        let destination = if self.output == "devopster-config.yaml" {
            config_path
        } else {
            &self.output
        };

        let sample = include_str!("../../devopster-config.example.yaml");

        if Path::new(destination).exists() {
            println!("Config already exists at {destination}");
            return Ok(());
        }

        std::fs::write(destination, sample)
            .with_context(|| format!("failed to write config to {destination}"))?;

        println!("Created configuration at {destination}");
        Ok(())
    }
}
