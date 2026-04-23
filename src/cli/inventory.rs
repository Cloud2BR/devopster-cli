use anyhow::Result;
use clap::Args;
use serde::Serialize;

use crate::config::AppConfig;
use crate::provider::{ProviderFactory, RepoSummary};
use crate::ui;

#[derive(Debug, Clone, Serialize)]
struct InventoryEntry {
    provider: String,
    organization: String,
    repository: RepoSummary,
}

#[derive(Debug, Args)]
pub struct InventoryCommand {
    /// Emit inventory as JSON
    #[arg(long)]
    pub json: bool,
}

impl InventoryCommand {
    pub async fn run(&self, config_path: &str) -> Result<()> {
        let config = AppConfig::load(config_path)?;
        let targets = config.provider_targets();
        let mut entries: Vec<InventoryEntry> = Vec::new();

        for target in targets {
            let provider = ProviderFactory::from_target(
                &config,
                &target.provider,
                target.project.as_deref(),
            )?;
            let repositories = provider.list_repositories(&target.organization).await?;
            let repositories = scoped_or_all(repositories, &config.scoped_repos);

            for repository in repositories {
                entries.push(InventoryEntry {
                    provider: target.provider.as_str().to_string(),
                    organization: target.organization.clone(),
                    repository,
                });
            }
        }

        entries.sort_by_key(|entry| {
            (
                entry.provider.clone(),
                entry.organization.clone(),
                entry.repository.name.to_lowercase(),
            )
        });

        if self.json {
            println!("{}", serde_json::to_string_pretty(&entries)?);
            return Ok(());
        }

        ui::header("Repository Inventory");
        ui::key_value("Targets", config.provider_targets().len());
        ui::key_value("Repositories", entries.len());
        if !config.scoped_repos.is_empty() {
            ui::note("Showing scoped repositories only.");
        }

        for entry in entries {
            let repo = entry.repository;
            let visibility = if repo.is_private { "private" } else { "public" };
            let branch = repo.default_branch.as_deref().unwrap_or("unknown-branch");
            let desc = if repo.description.trim().is_empty() {
                "(no description)"
            } else {
                repo.description.as_str()
            };

            ui::item(&format!(
                "{}/{} :: {} [{}] | branch: {} | topics: {} | {}",
                entry.provider,
                entry.organization,
                repo.name,
                visibility,
                branch,
                repo.topics.len(),
                desc
            ));
        }

        Ok(())
    }
}

fn scoped_or_all(repositories: Vec<RepoSummary>, scoped_repos: &[String]) -> Vec<RepoSummary> {
    if scoped_repos.is_empty() {
        return repositories;
    }

    repositories
        .into_iter()
        .filter(|repo| scoped_repos.iter().any(|name| name == &repo.name))
        .collect()
}
