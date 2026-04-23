use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use crate::ui;

#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Generate a configuration template from built-in defaults
    Template(TemplateCommand),
}

#[derive(Debug, Args)]
pub struct TemplateCommand {
    /// Output path for the generated template
    #[arg(long, default_value = "devopster-config.yaml")]
    pub output: String,

    /// Print template to stdout instead of writing a file
    #[arg(long, default_value_t = false)]
    pub stdout: bool,
}

impl ConfigCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            ConfigAction::Template(command) => command.run(),
        }
    }
}

impl TemplateCommand {
    pub fn run(&self) -> Result<()> {
        let template = default_config_template();

        if self.stdout {
            println!("{template}");
            return Ok(());
        }

        if let Some(parent) = Path::new(&self.output).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create directory for '{}'", self.output)
                })?;
            }
        }

        fs::write(&self.output, template)
            .with_context(|| format!("failed to write config template to '{}'", self.output))?;

        ui::success(&format!("Configuration template written to '{}'.", self.output));
        Ok(())
    }
}

fn default_config_template() -> &'static str {
    r#"provider: github
organization: your-org

# Optional multi-provider targets for cross-org inventory.
# When set, `devopster inventory` will aggregate these targets.
# providers:
#   - provider: github
#     organization: your-org
#   - provider: gitlab
#     organization: your-gitlab-group
#   - provider: azure_devops
#     organization: your-azure-org
#     project: your-project

# Limit devopster commands to specific repositories.
# Remove or leave empty to target all repositories in the organization.
# scoped_repos:
#   - my-service-repo
#   - platform-infra

# Enable GitHub Copilot-assisted suggestions for repos missing topics or descriptions.
# Requires a Copilot subscription. Set during `devopster init` or add manually.
# copilot_enabled: true

github:
  api_url: https://api.github.com
  token_env: GITHUB_TOKEN

azure_devops:
  organization_url: https://dev.azure.com/your-org
  project: your-project
  token_env: AZDO_TOKEN

gitlab:
  api_url: https://gitlab.com/api/v4
  token_env: GITLAB_TOKEN

default_branch: main
catalog:
  output_path: generated/catalog.json

# Audit policy — controls which checks `devopster repo audit` enforces.
# All rules default to enabled. Comment out or set to false to disable a check.
audit:
  require_description: true      # fail if the repo description is empty
  require_topics: true           # fail if the repo has fewer topics than min_topics
  min_topics: 1                  # minimum number of topics required (default: 1)
  require_license: true          # fail if no license is detected
  require_default_branch: true   # fail if the default branch does not match `default_branch`

# Blueprint source repository used by `devopster repo sync --from-blueprint`.
blueprint:
  repo: your-org/org-repo-template
  branch: main
  paths:
    - .github

templates:
  - name: azure-overview
    description: Base structure for Azure overview and demo repositories.
    topics:
      - azure
      - demo
      - overview
  - name: ai-agent
    description: Base structure for agent and AI repositories.
    topics:
      - ai
      - ai-agents
      - demo
"#
}
