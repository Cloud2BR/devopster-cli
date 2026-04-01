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

        let scoped = config.scoped_repos.len();
        let total = repos.len();

        let separator = "=".repeat(40);
        println!("{separator}");
        println!("  devopster stats");
        println!("{separator}");
        let w = 22usize;
        println!("{:<w$} {}", "  Organization:", config.organization);
        println!("{:<w$} {}", "  Provider:", config.provider.as_str());
        println!("{:<w$} {}", "  Repositories:", total);
        if scoped > 0 {
            println!("{:<w$} {} of {total}", "  Scoped to:", scoped);
        }
        if config.copilot_enabled {
            println!("{:<w$} enabled", "  Copilot:");
        }
        println!("{separator}");

        Ok(())
    }
}
