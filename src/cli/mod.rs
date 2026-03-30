pub mod catalog;
pub mod init;
pub mod repo;
pub mod stats;
pub mod topics;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "devopster",
    version,
    about = "Cross-platform GitOps CLI for managing organization repositories",
    long_about = "Manage GitHub, Azure DevOps, and GitLab organizations with a single containerized Rust CLI."
)]
pub struct Cli {
    #[arg(
        long,
        short = 'c',
        global = true,
        env = "DEVOPSTER_CONFIG",
        default_value = "devopster-config.yaml"
    )]
    pub config: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(init::InitCommand),
    Repo(repo::RepoCommand),
    Catalog(catalog::CatalogCommand),
    Topics(topics::TopicsCommand),
    Stats(stats::StatsCommand),
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(command) => command.run(&cli.config).await,
        Commands::Repo(command) => command.run(&cli.config).await,
        Commands::Catalog(command) => command.run(&cli.config).await,
        Commands::Topics(command) => command.run(&cli.config).await,
        Commands::Stats(command) => command.run(&cli.config).await,
    }
}
