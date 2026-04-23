use anyhow::Result;
use clap::Args;

use crate::cli::{init::InitCommand, login::LoginCommand, login::LoginProvider};
use crate::ui;

#[derive(Debug, Args)]
pub struct SetupCommand {
    #[arg(long, default_value = "devopster-config.yaml")]
    pub output: String,

    /// Sign in to all providers first, then continue with guided config setup
    #[arg(long)]
    pub login_all: bool,

    /// Skip login and only run the guided configuration flow
    #[arg(long)]
    pub no_login: bool,
}

impl SetupCommand {
    pub async fn run(&self, config_path: &str) -> Result<()> {
        ui::header("devopster setup");
        ui::note("Running end-to-end setup in one command.");

        if self.login_all && self.no_login {
            ui::warn("Both --login-all and --no-login were provided. Skipping login.");
        } else if self.login_all {
            ui::section("Sign in to all providers");
            LoginCommand {
                provider: LoginProvider::All,
            }
            .run()
            .await?;
        }

        ui::section("Create or update configuration");
        InitCommand {
            output: self.output.clone(),
            no_login: self.no_login || self.login_all,
        }
        .run(config_path)
        .await?;

        ui::section("Authentication summary");
        LoginCommand {
            provider: LoginProvider::Status,
        }
        .run()
        .await?;

        ui::success("Setup complete.");
        Ok(())
    }
}
