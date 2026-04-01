use std::io::{self, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::Args;

use crate::cli::login::{login_azure_devops, login_github, login_gitlab};

#[derive(Debug, Args)]
pub struct InitCommand {
    #[arg(long, default_value = "devopster-config.yaml")]
    pub output: String,

    /// Skip the interactive provider sign-in prompt
    #[arg(long)]
    pub no_login: bool,
}

impl InitCommand {
    pub async fn run(&self, config_path: &str) -> Result<()> {
        let destination = if self.output == "devopster-config.yaml" {
            config_path
        } else {
            &self.output
        };

        println!("Welcome to devopster. Let's set up your configuration.");
        println!();

        // ── Step 1: Pick provider ─────────────────────────────────────────────
        println!("Which provider would you like to configure?");
        println!("  1) GitHub");
        println!("  2) Azure DevOps");
        println!("  3) GitLab");
        print!("Choice [1]: ");
        io::stdout().flush().ok();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).context("failed to read provider choice")?;

        let provider = match choice.trim() {
            "2" | "azure" | "azure-devops" | "azure_devops" => "azure_devops",
            "3" | "gitlab" => "gitlab",
            _ => "github",
        };

        println!();

        // ── Step 2: Sign in ───────────────────────────────────────────────────
        if !self.no_login {
            let already_ok = is_authenticated(provider).await;
            if !already_ok {
                println!("Signing in to {}...", provider_display(provider));
                match provider {
                    "github" => login_github()?,
                    "azure_devops" => login_azure_devops()?,
                    "gitlab" => login_gitlab()?,
                    _ => {}
                }
                println!();
            } else {
                println!("Already signed in to {}.", provider_display(provider));
                println!();
            }
        }

        // ── Step 3: Pick org / namespace ──────────────────────────────────────
        let (org, project, api_url) = pick_org(provider).await?;
        println!("  Using: {org}");
        println!();

        // ── Step 4: Pick repositories to target ──────────────────────────────
        let scoped_repos = pick_repos(provider, &org, project.as_deref(), &api_url).await;
        println!();

        // ── Step 5: Copilot (GitHub only) ─────────────────────────────────────
        let copilot_enabled = if provider == "github" {
            ask_copilot_enabled().await
        } else {
            false
        };
        println!();

        // ── Step 6: Write config ──────────────────────────────────────────────
        let yaml = build_config_yaml(
            provider,
            &org,
            project.as_deref(),
            &api_url,
            &scoped_repos,
            copilot_enabled,
        );

        if Path::new(destination).exists() {
            let existing = std::fs::read_to_string(destination).unwrap_or_default();
            if existing == yaml {
                println!("Configuration is already up to date at {destination}.");
                println!();
                println!("Run `devopster repo list` to get started.");
                return Ok(());
            }

            // Show what will change.
            println!("Configuration summary (will be saved to {destination}):");
            println!();
            print_config_summary(provider, &org, project.as_deref(), &scoped_repos, copilot_enabled);

            // Show what is currently in the file so the user can see what they'd lose.
            println!();
            println!("Existing file contents:");
            println!("  {}", existing.lines().take(8).collect::<Vec<_>>().join("\n  "));
            if existing.lines().count() > 8 {
                println!("  ... ({} more lines)", existing.lines().count().saturating_sub(8));
            }
            println!();
            print!("Apply these values? [Y/n]: ");
            io::stdout().flush().ok();
            let mut ow = String::new();
            io::stdin().read_line(&mut ow).ok();
            let answer = ow.trim();
            if answer.eq_ignore_ascii_case("n") {
                println!("Keeping existing config.");
                return Ok(());
            }
        } else {
            println!("Configuration summary (will be saved to {destination}):");
            println!();
            print_config_summary(provider, &org, project.as_deref(), &scoped_repos, copilot_enabled);
            println!();
            print!("Save this configuration? [Y/n]: ");
            io::stdout().flush().ok();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).ok();
            if confirm.trim().eq_ignore_ascii_case("n") {
                println!("Cancelled.");
                return Ok(());
            }
        }

        std::fs::write(destination, &yaml)
            .with_context(|| format!("failed to write config to {destination}"))?;

        println!();
        println!("Configuration saved to {destination}.");
        println!("Run `devopster repo list` to get started.");

        Ok(())
    }
}

// ── Provider helpers ─────────────────────────────────────────────────────────

fn provider_display(provider: &str) -> &str {
    match provider {
        "github" => "GitHub",
        "azure_devops" => "Azure DevOps",
        "gitlab" => "GitLab",
        _ => provider,
    }
}

async fn is_authenticated(provider: &str) -> bool {
    match provider {
        "github" => cli_ok("gh", &["auth", "status"]).await,
        "azure_devops" => cli_ok("az", &["account", "show"]).await,
        "gitlab" => cli_ok("glab", &["auth", "status"]).await,
        _ => false,
    }
}

// ── Org / namespace picker ────────────────────────────────────────────────────

/// Returns `(org_name, optional_project, api_url)`.
async fn pick_org(provider: &str) -> Result<(String, Option<String>, String)> {
    match provider {
        "github" => pick_github_org().await,
        "azure_devops" => pick_azure_org().await,
        "gitlab" => pick_gitlab_group().await,
        _ => bail!("unknown provider: {provider}"),
    }
}

async fn pick_github_org() -> Result<(String, Option<String>, String)> {
    println!("Fetching your GitHub accounts and organizations...");

    let mut orgs: Vec<String> = Vec::new();

    // Personal account namespace
    if let Ok(login) = cli_capture("gh", &["api", "/user", "--jq", ".login"]).await {
        let login = login.trim().to_string();
        if !login.is_empty() {
            orgs.push(format!("{login} (personal)"));
        }
    }

    // Organizations the user belongs to
    if let Ok(out) = cli_capture(
        "gh",
        &["api", "/user/orgs", "--paginate", "--jq", ".[].login"],
    )
    .await
    {
        for line in out.lines() {
            let l = line.trim().to_string();
            if !l.is_empty() {
                orgs.push(l);
            }
        }
    }

    if orgs.is_empty() {
        return ask_org_url("github");
    }

    println!("Available accounts and organizations:");
    for (i, org) in orgs.iter().enumerate() {
        println!("  {}) {}", i + 1, org);
    }
    print!("Choice [1] or paste a GitHub URL: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).context("failed to read org choice")?;
    let trimmed = input.trim();

    if trimmed.starts_with("http") {
        let org = last_url_segment(trimmed)?;
        return Ok((org, None, "https://api.github.com".to_string()));
    }

    let idx = parse_index_or_default(trimmed, 1).saturating_sub(1);
    let selected = orgs.get(idx).unwrap_or(&orgs[0]).clone();
    let org = selected.trim_end_matches(" (personal)").to_string();
    Ok((org, None, "https://api.github.com".to_string()))
}

async fn pick_azure_org() -> Result<(String, Option<String>, String)> {
    println!("Enter your Azure DevOps organization URL");
    println!("  e.g. https://dev.azure.com/my-org");
    print!("URL: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).context("failed to read Azure DevOps org URL")?;
    let org_url = input.trim().trim_end_matches('/').to_string();

    let org_name = last_url_segment(&org_url)
        .context("could not extract org name from Azure DevOps URL")?;

    // List projects under this org
    println!("Fetching projects for {}...", org_url);
    let projects: Vec<String> = cli_capture(
        "az",
        &[
            "devops",
            "project",
            "list",
            "--org",
            &org_url,
            "--output",
            "json",
            "--query",
            "[].name",
        ],
    )
    .await
    .ok()
    .and_then(|json| serde_json::from_str(&json).ok())
    .unwrap_or_default();

    let project = if projects.is_empty() {
        prompt("Azure DevOps project name")?
    } else if projects.len() == 1 {
        projects.into_iter().next().unwrap()
    } else {
        println!("Available projects:");
        for (i, p) in projects.iter().enumerate() {
            println!("  {}) {}", i + 1, p);
        }
        print!("Choice [1]: ");
        io::stdout().flush().ok();
        let mut p_input = String::new();
        io::stdin().read_line(&mut p_input).context("failed to read project choice")?;
        let idx = parse_index_or_default(p_input.trim(), 1).saturating_sub(1);
        projects.get(idx).unwrap_or(&projects[0]).clone()
    };

    Ok((org_name, Some(project), org_url))
}

async fn pick_gitlab_group() -> Result<(String, Option<String>, String)> {
    println!("Fetching your GitLab namespaces and groups...");

    let mut groups: Vec<String> = Vec::new();

    // Personal namespace
    if let Ok(username) = cli_capture("glab", &["api", "/user", "-q", ".username"]).await {
        let username = username.trim().to_string();
        if !username.is_empty() {
            groups.push(format!("{username} (personal)"));
        }
    }

    // Groups the user is a member of
    if let Ok(out) = cli_capture(
        "glab",
        &["api", "/groups", "--field", "per_page=100", "-q", ".[].full_path"],
    )
    .await
    {
        for line in out.lines() {
            let l = line.trim().to_string();
            if !l.is_empty() {
                groups.push(l);
            }
        }
    }

    if groups.is_empty() {
        return ask_org_url("gitlab");
    }

    println!("Available namespaces and groups:");
    for (i, g) in groups.iter().enumerate() {
        println!("  {}) {}", i + 1, g);
    }
    print!("Choice [1] or paste a GitLab URL: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).context("failed to read group choice")?;
    let trimmed = input.trim();

    if trimmed.starts_with("http") {
        let group = last_url_segment(trimmed)?;
        return Ok((group, None, "https://gitlab.com/api/v4".to_string()));
    }

    let idx = parse_index_or_default(trimmed, 1).saturating_sub(1);
    let selected = groups.get(idx).unwrap_or(&groups[0]).clone();
    let group = selected.trim_end_matches(" (personal)").to_string();
    Ok((group, None, "https://gitlab.com/api/v4".to_string()))
}

fn ask_org_url(provider: &str) -> Result<(String, Option<String>, String)> {
    let example = match provider {
        "github" => "https://github.com/my-org",
        "gitlab" => "https://gitlab.com/my-group",
        _ => "https://dev.azure.com/my-org",
    };
    println!("Could not fetch organizations automatically.");
    println!("Paste your organization URL (e.g. {example}):");
    print!("URL: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).context("failed to read org URL")?;
    let url_owned = input.trim().trim_end_matches('/').to_string();

    let org = last_url_segment(&url_owned)?;
    let api_url = match provider {
        "github" => "https://api.github.com".to_string(),
        "gitlab" => "https://gitlab.com/api/v4".to_string(),
        _ => {
            let parts: Vec<&str> = url_owned.splitn(4, '/').collect();
            parts[..parts.len().min(3)].join("/")
        }
    };
    Ok((org, None, api_url))
}

// ── Repo scope picker ─────────────────────────────────────────────────────────

async fn pick_repos(
    provider: &str,
    org: &str,
    project: Option<&str>,
    org_url: &str,
) -> Vec<String> {
    let repos = fetch_repo_names(provider, org, project, org_url).await;

    let repos = match repos {
        Ok(r) if !r.is_empty() => r,
        _ => {
            println!("  Could not fetch repository list -- all repos will be targeted.");
            return Vec::new();
        }
    };

    println!("Found {} repositories in {}.", repos.len(), org);
    println!("  a) Target all {} repositories", repos.len());
    println!("  s) Select specific repositories");
    print!("Choice [a]: ");
    io::stdout().flush().ok();

    let mut scope_input = String::new();
    if io::stdin().read_line(&mut scope_input).is_err() {
        return Vec::new();
    }

    if !scope_input.trim().eq_ignore_ascii_case("s") {
        return Vec::new(); // empty = all
    }

    println!();
    for (i, r) in repos.iter().enumerate() {
        println!("  {:>3}) {}", i + 1, r);
    }
    println!();
    print!("Enter numbers (comma-separated) or 'a' for all: ");
    io::stdout().flush().ok();

    let mut sel = String::new();
    if io::stdin().read_line(&mut sel).is_err() {
        return Vec::new();
    }

    let trimmed = sel.trim();
    if trimmed.eq_ignore_ascii_case("a") {
        return Vec::new();
    }

    let selected: Vec<String> = trimmed
        .split(',')
        .filter_map(|s| {
            let idx = s.trim().parse::<usize>().ok()?.saturating_sub(1);
            repos.get(idx).cloned()
        })
        .collect();

    if selected.is_empty() {
        return Vec::new();
    }

    println!("  {} repositories selected.", selected.len());
    selected
}

async fn fetch_repo_names(
    provider: &str,
    org: &str,
    project: Option<&str>,
    org_url: &str,
) -> Result<Vec<String>> {
    match provider {
        "github" => {
            let out = cli_capture(
                "gh",
                &[
                    "repo", "list", org,
                    "--limit", "200",
                    "--json", "name",
                    "--jq", ".[].name",
                ],
            )
            .await?;
            Ok(non_empty_lines(&out))
        }
        "azure_devops" => {
            let project = project.context("Azure DevOps project is required")?;
            let json = cli_capture(
                "az",
                &[
                    "repos", "list",
                    "--org", org_url,
                    "--project", project,
                    "--output", "json",
                    "--query", "[].name",
                ],
            )
            .await?;
            Ok(serde_json::from_str::<Vec<String>>(&json)?)
        }
        "gitlab" => {
            let encoded = org.replace('/', "%2F");
            let endpoint =
                format!("/groups/{encoded}/projects?per_page=100&simple=true");
            let out = cli_capture("glab", &["api", &endpoint, "-q", ".[].path"]).await?;
            Ok(non_empty_lines(&out))
        }
        _ => bail!("unknown provider: {provider}"),
    }
}

// ── Copilot prompt ────────────────────────────────────────────────────────────

async fn ask_copilot_enabled() -> bool {
    print!("Enable Copilot-assisted suggestions for repos missing topics or descriptions? \
            (Requires a GitHub Copilot subscription.) [Y/n]: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return true;
    }
    let trimmed = input.trim();
    // Default is Y — only 'n' or 'no' disables it.
    trimmed.is_empty() || !trimmed.eq_ignore_ascii_case("n")
}

// ── Config summary display ────────────────────────────────────────────────────

fn print_config_summary(
    provider: &str,
    org: &str,
    project: Option<&str>,
    scoped_repos: &[String],
    copilot_enabled: bool,
) {
    let w = 18usize;
    println!("  {:<w$} {}", "Provider:", provider_display(provider));
    println!("  {:<w$} {}", "Organization:", org);
    if let Some(p) = project {
        println!("  {:<w$} {}", "Project:", p);
    }
    if scoped_repos.is_empty() {
        println!("  {:<w$} all repositories", "Scope:");
    } else {
        println!("  {:<w$} {} selected: {}", "Scope:", scoped_repos.len(), scoped_repos.join(", "));
    }
    println!("  {:<w$} {}", "Copilot:", if copilot_enabled { "enabled" } else { "disabled" });
}

// ── Config YAML builder ───────────────────────────────────────────────────────

fn build_config_yaml(
    provider: &str,
    org: &str,
    project: Option<&str>,
    org_url: &str,
    scoped_repos: &[String],
    copilot_enabled: bool,
) -> String {
    let mut y = String::new();

    y.push_str(&format!("provider: {provider}\n"));
    y.push_str(&format!("organization: {org}\n"));

    if !scoped_repos.is_empty() {
        y.push_str("scoped_repos:\n");
        for r in scoped_repos {
            y.push_str(&format!("  - {r}\n"));
        }
    }

    if copilot_enabled {
        y.push_str("copilot_enabled: true\n");
    }

    y.push('\n');

    match provider {
        "github" => {
            y.push_str("github:\n");
            y.push_str("  api_url: https://api.github.com\n");
            y.push_str("  token_env: GITHUB_TOKEN\n");
        }
        "azure_devops" => {
            let proj = project.unwrap_or("your-project");
            y.push_str("azure_devops:\n");
            y.push_str(&format!("  organization_url: {org_url}\n"));
            y.push_str(&format!("  project: {proj}\n"));
            y.push_str("  token_env: AZDO_TOKEN\n");
        }
        "gitlab" => {
            y.push_str("gitlab:\n");
            y.push_str("  api_url: https://gitlab.com/api/v4\n");
            y.push_str("  token_env: GITLAB_TOKEN\n");
        }
        _ => {}
    }

    y.push('\n');
    y.push_str("default_branch: main\n");
    y.push_str("catalog:\n");
    y.push_str("  output_path: generated/catalog.json\n");
    y.push('\n');
    y.push_str("templates:\n");
    y.push_str("  - name: default\n");
    y.push_str("    description: Default repository template.\n");
    y.push_str("    topics: []\n");

    y
}

// ── Shared CLI utilities ──────────────────────────────────────────────────────

/// Run a command and return true if it exits successfully.
async fn cli_ok(bin: &str, args: &[&str]) -> bool {
    tokio::process::Command::new(bin)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run a command and return its trimmed stdout, or an error.
async fn cli_capture(bin: &str, args: &[&str]) -> Result<String> {
    let output = tokio::process::Command::new(bin)
        .args(args)
        .output()
        .await
        .with_context(|| format!("failed to run `{bin}`"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`{bin}` exited with an error: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn non_empty_lines(s: &str) -> Vec<String> {
    s.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn last_url_segment(url: &str) -> Result<String> {
    url.trim_end_matches('/')
        .split('/')
        .last()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .context("could not extract name from URL")
}

fn parse_index_or_default(s: &str, default: usize) -> usize {
    if s.is_empty() {
        default
    } else {
        s.parse::<usize>().unwrap_or(default)
    }
}

fn prompt(label: &str) -> Result<String> {
    print!("{label}: ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).context("failed to read input")?;
    Ok(input.trim().to_string())
}
