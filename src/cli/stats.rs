use anyhow::Result;
use clap::Args;

use crate::config::AppConfig;
use crate::provider::ProviderFactory;

#[derive(Debug, Args)]
pub struct StatsCommand {}

impl StatsCommand {
    pub async fn run(&self, config_path: &str) -> Result<()> {
        let config = AppConfig::load(config_path)?;
        let provider = ProviderFactory::from_config(&config)?;
        let repos = provider.list_repositories(&config.organization).await?;

        println!("Organization: {}", config.organization);
        println!("Provider: {}", config.provider.as_str());
        println!("Repositories discovered: {}", repos.len());

        Ok(())
    }
}
