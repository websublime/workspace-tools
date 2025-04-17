use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::{env, fs};
use sublime_workspace_cli::common::config::{get_config_path, Config, RepositoryConfig};
use sublime_workspace_cli::ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize or reset the configuration file
    Init(InitOptions),

    /// Print the current configuration file path
    Path,

    /// View the current configuration
    View {
        /// Show the raw TOML format
        #[arg(short, long)]
        raw: bool,
    },

    /// Edit the configuration file with your default editor
    Edit,

    /// Get a specific configuration value
    Get {
        /// The configuration key (e.g., "daemon.socket_path")
        key: String,
    },

    /// Set a specific configuration value
    Set {
        /// The configuration key (e.g., "daemon.socket_path")
        key: String,

        /// The configuration value
        value: String,
    },

    /// Add a repository to the configuration
    AddRepo(AddRepoOptions),

    /// Remove a repository from the configuration
    RemoveRepo {
        /// Repository name or path
        identifier: String,
    },

    /// List all configured repositories
    ListRepos,
}

#[derive(Args)]
struct InitOptions {
    /// Force overwrite if configuration already exists
    #[arg(short, long)]
    force: bool,

    /// Path to save configuration (defaults to standard location)
    #[arg(short, long)]
    path: Option<PathBuf>,
}

#[derive(Args)]
struct AddRepoOptions {
    /// Path to the repository
    #[arg(short, long)]
    path: PathBuf,

    /// Name for the repository (defaults to directory name)
    #[arg(short, long)]
    name: Option<String>,

    /// Whether the repository is active (defaults to true)
    #[arg(short, long)]
    active: Option<bool>,

    /// Main branch for the repository
    #[arg(short, long)]
    branch: Option<String>,
}

fn main() -> Result<()> {
    // Initialize the UI system
    ui::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init(options)) => init_config(&options)?,
        Some(Commands::Path) => {
            let path = get_config_path();
            println!("{}", ui::file_path(&path.to_string_lossy()));
        }
        Some(Commands::View { raw }) => view_config(raw)?,
        Some(Commands::Edit) => edit_config()?,
        Some(Commands::Get { key }) => get_config_value(&key)?,
        Some(Commands::Set { key, value }) => set_config_value(&key, &value)?,
        Some(Commands::AddRepo(options)) => add_repo(&options)?,
        Some(Commands::RemoveRepo { identifier }) => remove_repo(&identifier)?,
        Some(Commands::ListRepos) => list_repos()?,
        None => {
            println!("{}", ui::section_header("Workspace Configuration Tool"));
            println!("Use one of the following commands:");
            println!(
                "{}",
                ui::command_example("workspace config init    - Initialize configuration")
            );
            println!("{}", ui::command_example("workspace config path    - Show config file path"));
            println!(
                "{}",
                ui::command_example("workspace config view    - View current configuration")
            );
            println!(
                "{}",
                ui::command_example("workspace config edit    - Edit configuration file")
            );
            println!("{}", ui::command_example("workspace config get     - Get a specific value"));
            println!("{}", ui::command_example("workspace config set     - Set a specific value"));
            println!("{}", ui::command_example("workspace config addrepo - Add a repository"));
            println!("{}", ui::command_example("workspace config remrepo - Remove a repository"));
            println!("{}", ui::command_example("workspace config listrepo - List repositories"));
            println!("\nFor more details, run 'workspace config --help'");
        }
    }

    Ok(())
}

fn init_config(options: &InitOptions) -> Result<()> {
    let path = options.path.clone().unwrap_or_else(get_config_path);

    // Check if config already exists
    if path.exists() && !options.force {
        println!("{}", ui::warning("Configuration file already exists. Use --force to overwrite."));
        println!("Path: {}", ui::file_path(&path.to_string_lossy()));
        return Ok(());
    }

    // Create default config
    let config = Config::default();

    // Save the config
    let content = toml::to_string_pretty(&config).context("Failed to serialize config to TOML")?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    fs::write(&path, content).context("Failed to write config file")?;

    println!("{}", ui::success("Configuration initialized successfully!"));
    println!("Path: {}", ui::file_path(&path.to_string_lossy()));

    Ok(())
}

fn view_config(raw: bool) -> Result<()> {
    let path = get_config_path();

    if !path.exists() {
        println!("{}", ui::warning("Configuration file does not exist."));
        println!("Run 'workspace config init' to create it.");
        return Ok(());
    }

    let config = Config::load(&path).context("Failed to load configuration")?;

    if raw {
        let content = fs::read_to_string(&path).context("Failed to read config file")?;
        println!("{}", content);
    } else {
        println!("{}", ui::section_header("Workspace Configuration"));

        if let Some(general) = &config.general {
            println!("{}", ui::highlight("General Settings"));
            if let Some(level) = &general.log_level {
                println!("{}", ui::key_value("Log Level", level));
            }
            if let Some(auto_start) = general.auto_start_daemon {
                println!("{}", ui::key_value("Auto-start Daemon", &auto_start.to_string()));
            }
            println!();
        }

        if let Some(daemon) = &config.daemon {
            println!("{}", ui::highlight("Daemon Settings"));
            if let Some(socket) = &daemon.socket_path {
                println!("{}", ui::key_value("Socket Path", socket));
            }
            if let Some(pid) = &daemon.pid_file {
                println!("{}", ui::key_value("PID File", pid));
            }
            if let Some(interval) = daemon.polling_interval_ms {
                println!("{}", ui::key_value("Polling Interval (ms)", &interval.to_string()));
            }
            if let Some(inactive) = daemon.inactive_polling_ms {
                println!("{}", ui::key_value("Inactive Polling (ms)", &inactive.to_string()));
            }
            println!();
        }

        if let Some(monitor) = &config.monitor {
            println!("{}", ui::highlight("Monitor Settings"));
            if let Some(refresh) = monitor.refresh_rate_ms {
                println!("{}", ui::key_value("Refresh Rate (ms)", &refresh.to_string()));
            }
            if let Some(view) = &monitor.default_view {
                println!("{}", ui::key_value("Default View", view));
            }
            if let Some(theme) = &monitor.color_theme {
                println!("{}", ui::key_value("Color Theme", theme));
            }
            println!();
        }

        if let Some(watcher) = &config.watcher {
            println!("{}", ui::highlight("Watcher Settings"));
            if let Some(include) = &watcher.include_patterns {
                println!(
                    "{}",
                    ui::key_value("Include Patterns", &format!("[{} patterns]", include.len()))
                );
                for pattern in include {
                    println!("  - {}", pattern);
                }
            }
            if let Some(exclude) = &watcher.exclude_patterns {
                println!(
                    "{}",
                    ui::key_value("Exclude Patterns", &format!("[{} patterns]", exclude.len()))
                );
                for pattern in exclude {
                    println!("  - {}", pattern);
                }
            }
            if let Some(git_hooks) = watcher.use_git_hooks {
                println!("{}", ui::key_value("Use Git Hooks", &git_hooks.to_string()));
            }
            println!();
        }

        if let Some(github) = &config.github {
            println!("{}", ui::highlight("GitHub Integration"));
            if let Some(enabled) = github.enable_integration {
                println!("{}", ui::key_value("Enabled", &enabled.to_string()));
            }
            if let Some(token_path) = &github.token_path {
                println!("{}", ui::key_value("Token Path", token_path));
            }
            if let Some(interval) = github.fetch_interval_s {
                println!("{}", ui::key_value("Fetch Interval (s)", &interval.to_string()));
            }
            println!();
        }

        if let Some(repos) = &config.repositories {
            println!("{}", ui::highlight("Repositories"));
            if repos.is_empty() {
                println!("No repositories configured.");
            } else {
                for (i, repo) in repos.iter().enumerate() {
                    // Create a longer-lived value to avoid the temporary being dropped
                    let default_name = format!("Repository {}", i + 1);
                    let name = repo.name.as_ref().unwrap_or(&default_name);
                    println!("{}", ui::key_value(name, &repo.path));

                    if let Some(active) = repo.active {
                        println!("  Active: {}", active);
                    }

                    if let Some(branch) = &repo.branch {
                        println!("  Branch: {}", branch);
                    }

                    println!();
                }
            }
        }
    }

    Ok(())
}

fn edit_config() -> Result<()> {
    let path = get_config_path();

    if !path.exists() {
        println!("{}", ui::warning("Configuration file does not exist."));
        println!("Run 'workspace config init' to create it.");
        return Ok(());
    }

    let editor = env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(windows) {
            "notepad".to_string()
        } else {
            "vim".to_string()
        }
    });

    println!("{}", ui::info(&format!("Opening configuration with {}", editor)));

    let status = std::process::Command::new(editor)
        .arg(&path)
        .status()
        .context("Failed to launch editor")?;

    if status.success() {
        println!("{}", ui::success("Editor closed. Configuration updated."));
    } else {
        println!("{}", ui::error("Editor exited with an error."));
    }

    Ok(())
}

fn get_config_value(key: &str) -> Result<()> {
    let path = get_config_path();

    if !path.exists() {
        println!("{}", ui::warning("Configuration file does not exist."));
        println!("Run 'workspace config init' to create it.");
        return Ok(());
    }

    let content = fs::read_to_string(&path).context("Failed to read config file")?;
    let config: toml::Value =
        toml::from_str(&content).context("Failed to parse configuration TOML")?;

    let keys: Vec<&str> = key.split('.').collect();
    let mut current = &config;

    for (i, k) in keys.iter().enumerate() {
        if let Some(value) = current.get(*k) {
            current = value;
            if i == keys.len() - 1 {
                println!("{}: {}", ui::primary_style(key), current);
            }
        } else {
            println!("{}", ui::error(&format!("Key '{}' not found in configuration", key)));
            return Ok(());
        }
    }

    Ok(())
}

fn set_config_value(key: &str, value: &str) -> Result<()> {
    let path = get_config_path();

    if !path.exists() {
        println!("{}", ui::warning("Configuration file does not exist."));
        println!("Run 'workspace config init' to create it.");
        return Ok(());
    }

    let content = fs::read_to_string(&path).context("Failed to read config file")?;
    let mut config: toml::Value =
        toml::from_str(&content).context("Failed to parse configuration TOML")?;

    let keys: Vec<&str> = key.split('.').collect();

    // The value needs to be parsed according to the target type
    let parsed_value = if value.eq_ignore_ascii_case("true") {
        toml::Value::Boolean(true)
    } else if value.eq_ignore_ascii_case("false") {
        toml::Value::Boolean(false)
    } else if let Ok(int_val) = value.parse::<i64>() {
        toml::Value::Integer(int_val)
    } else if let Ok(float_val) = value.parse::<f64>() {
        toml::Value::Float(float_val)
    } else {
        toml::Value::String(value.to_string())
    };

    // Navigate to the right spot in the config
    let mut current = &mut config;
    for (i, k) in keys.iter().enumerate() {
        if i == keys.len() - 1 {
            // Last key, set the value
            current[*k] = parsed_value.clone();
        } else {
            // Ensure this part of the path exists
            if current.get(*k).is_none() {
                current[*k] = toml::Value::Table(toml::map::Map::new());
            }
            current = current.get_mut(*k).unwrap();
            if !current.is_table() {
                return Err(anyhow::anyhow!("Cannot set value: '{}' is not a table", k));
            }
        }
    }

    // Write back the modified config
    let updated_content =
        toml::to_string_pretty(&config).context("Failed to serialize config to TOML")?;
    fs::write(&path, updated_content).context("Failed to write config file")?;

    println!("{}", ui::success(&format!("Configuration updated: {} = {}", key, value)));

    Ok(())
}

fn add_repo(options: &AddRepoOptions) -> Result<()> {
    let path = get_config_path();

    // Create config if it doesn't exist
    if !path.exists() {
        println!("{}", ui::info("Configuration file does not exist, creating it..."));
        init_config(&InitOptions { force: false, path: None })?;
    }

    // Read config
    let mut config = Config::load(&path).context("Failed to load configuration")?;

    // Create repositories section if it doesn't exist
    if config.repositories.is_none() {
        config.repositories = Some(Vec::new());
    }

    // Generate a default name if none provided
    let name = options.name.clone().unwrap_or_else(|| {
        options.path.file_name().and_then(|n| n.to_str()).unwrap_or("unnamed").to_string()
    });

    // Check if a repository with this name or path already exists
    let repos = config.repositories.as_ref().unwrap();
    if repos
        .iter()
        .any(|r| r.path == options.path.to_string_lossy() || r.name.as_ref() == Some(&name))
    {
        println!("{}", ui::error("A repository with this name or path already exists."));
        return Ok(());
    }

    // Create new repository entry
    let repo = RepositoryConfig {
        path: options.path.to_string_lossy().to_string(),
        name: Some(name.clone()),
        active: options.active.or(Some(true)),
        branch: options.branch.clone(),
        include_patterns: None,
        exclude_patterns: None,
    };

    // Add to config
    config.repositories.as_mut().unwrap().push(repo);

    // Write back config
    let content = toml::to_string_pretty(&config).context("Failed to serialize config to TOML")?;
    fs::write(&path, content).context("Failed to write config file")?;

    println!("{}", ui::success(&format!("Repository '{}' added to configuration.", name)));

    Ok(())
}

fn remove_repo(identifier: &str) -> Result<()> {
    let path = get_config_path();

    if !path.exists() {
        println!("{}", ui::warning("Configuration file does not exist."));
        println!("Run 'workspace config init' to create it.");
        return Ok(());
    }

    // Read config
    let mut config = Config::load(&path).context("Failed to load configuration")?;

    // Check if repositories section exists
    if config.repositories.is_none() || config.repositories.as_ref().unwrap().is_empty() {
        println!("{}", ui::warning("No repositories are configured."));
        return Ok(());
    }

    // Find repository by name or path
    let repos = config.repositories.as_mut().unwrap();
    let initial_len = repos.len();

    // Remove matching repositories
    repos.retain(|r| r.name.as_ref() != Some(&identifier.to_string()) && r.path != identifier);

    // Check if any were removed
    if repos.len() == initial_len {
        println!(
            "{}",
            ui::error(&format!("No repository found with name or path '{}'", identifier))
        );
        return Ok(());
    }

    // Write back config
    let content = toml::to_string_pretty(&config).context("Failed to serialize config to TOML")?;
    fs::write(&path, content).context("Failed to write config file")?;

    println!(
        "{}",
        ui::success(&format!("Repository '{}' removed from configuration.", identifier))
    );

    Ok(())
}

fn list_repos() -> Result<()> {
    let path = get_config_path();

    if !path.exists() {
        println!("{}", ui::warning("Configuration file does not exist."));
        println!("Run 'workspace config init' to create it.");
        return Ok(());
    }

    // Read config
    let config = Config::load(&path).context("Failed to load configuration")?;

    // Check if repositories section exists
    if config.repositories.is_none() || config.repositories.as_ref().unwrap().is_empty() {
        println!("{}", ui::warning("No repositories are configured."));
        return Ok(());
    }

    // Display repositories
    println!("{}", ui::section_header("Configured Repositories"));

    let repos = config.repositories.as_ref().unwrap();
    let mut table_rows = Vec::new();

    for repo in repos {
        // Create a longer-lived value for the default name
        let unnamed = "unnamed".to_string();
        let name = repo.name.as_ref().unwrap_or(&unnamed);
        let active = repo.active.unwrap_or(true);

        // Create a longer-lived value for the default branch
        let default_branch = "default".to_string();
        let branch = repo.branch.as_ref().unwrap_or(&default_branch);

        table_rows.push(vec![
            name.to_string(),
            repo.path.clone(),
            active.to_string(),
            branch.to_string(),
        ]);
    }

    let headers =
        vec!["Name".to_string(), "Path".to_string(), "Active".to_string(), "Branch".to_string()];
    println!("{}", ui::create_table(headers, table_rows));

    Ok(())
}
