use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::config::AppConfig;
use crate::provider::{ProviderFactory, RepoSummary};

#[derive(Debug, Args)]
pub struct CatalogCommand {
    #[command(subcommand)]
    pub action: CatalogAction,
}

#[derive(Debug, Subcommand)]
pub enum CatalogAction {
    Generate(GenerateCatalogCommand),
}

#[derive(Debug, Args)]
pub struct GenerateCatalogCommand {}

#[derive(Serialize)]
struct Catalog {
    organization: String,
    provider: String,
    repository_count: usize,
    repositories: Vec<RepoSummary>,
}

impl CatalogCommand {
    pub async fn run(&self, config_path: &str) -> Result<()> {
        let config = AppConfig::load(config_path)?;
        let provider = ProviderFactory::from_config(&config)?;

        match &self.action {
            CatalogAction::Generate(_) => {
                let repositories = provider.list_repositories(&config.organization).await?;
                let count = repositories.len();

                let catalog = Catalog {
                    organization: config.organization.clone(),
                    provider: config.provider.as_str().to_string(),
                    repository_count: count,
                    repositories,
                };

                let json = serde_json::to_string_pretty(&catalog)
                    .map_err(|e| anyhow::anyhow!("failed to serialize catalog: {e}"))?;

                if let Some(parent) =
                    std::path::Path::new(&config.catalog.output_path).parent()
                {
                    if !parent.as_os_str().is_empty() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            anyhow::anyhow!("failed to create output directory: {e}")
                        })?;
                    }
                }

                std::fs::write(&config.catalog.output_path, json).map_err(|e| {
                    anyhow::anyhow!(
                        "failed to write catalog to '{}': {e}",
                        config.catalog.output_path
                    )
                })?;

                println!(
                    "Catalog written to '{}': {count} repositories.",
                    config.catalog.output_path
                );
            }
        }

        Ok(())
    }
}
