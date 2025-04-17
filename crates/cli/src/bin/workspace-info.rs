use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use sublime_git_tools::Repo;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
use sublime_package_tools::ValidationOptions;
use sublime_workspace_cli::{
    common::config::{get_config_path, Config, RepositoryConfig},
    common::workspace_detection,
    ui,
};

#[derive(Parser)]
#[command(name = "workspace-info")]
#[command(author = "Sublime")]
#[command(version)]
#[command(
    about = "Displays information about your workspace including packages, dependencies, and Git status"
)]
#[command(
    long_about = "A detailed workspace analysis tool that provides information about your monorepo structure, \
packages, dependencies, Git status, and can visualize dependency graphs."
)]
struct Cli {
    /// Path to the workspace (defaults to current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Repository name from config (overrides path)
    #[arg(short, long)]
    name: Option<String>,

    /// Show detailed information about each package
    #[arg(short, long)]
    verbose: bool,

    /// Output as JSON for programmatic use
    #[arg(short, long)]
    json: bool,

    /// Show dependency graph
    #[arg(short = 'g', long)]
    graph: bool,

    /// Show only circular dependencies
    #[arg(short = 'y', long)]
    cycles: bool,

    /// Show dependency validation results
    #[arg(short = 'd', long)]
    validate: bool,
}

fn main() -> Result<()> {
    // Initialize the UI system
    ui::init();

    let cli = Cli::parse();

    // Set up logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Load config
    let config_path = get_config_path();
    let config = match Config::load(&config_path) {
        Ok(config) => config,
        Err(e) => {
            log::warn!("Failed to load config from {}: {}", config_path.display(), e);
            Config::default()
        }
    };

    // Determine repositories to analyze
    let repositories = determine_repositories_to_analyze(&cli, &config)?;

    if repositories.is_empty() {
        println!("{}", ui::error("No repositories found to analyze"));
        return Ok(());
    }

    // Process each repository
    if cli.json {
        output_json_for_repositories(
            &repositories,
            cli.verbose,
            cli.graph,
            cli.cycles,
            cli.validate,
        )?;
    } else {
        for repo in repositories.iter() {
            output_human_readable_for_repository(repo, cli.verbose, cli.graph, cli.validate)?;
        }
    }

    Ok(())
}

/// Determine which repositories to analyze based on CLI args and config
fn determine_repositories_to_analyze(cli: &Cli, config: &Config) -> Result<Vec<RepositoryConfig>> {
    match (cli.path.as_ref(), cli.name.as_ref()) {
        // Case 1: Specific path provided - create a temporary repo config
        (Some(path), None) => Ok(vec![RepositoryConfig {
            path: path.to_string_lossy().to_string(),
            name: None,
            active: Some(true),
            branch: None,
            include_patterns: None,
            exclude_patterns: None,
        }]),
        // Case 2: Specific name provided - find matching repo in config
        (None, Some(name)) => {
            if let Some(repos) = &config.repositories {
                let matching_repos = repos
                    .iter()
                    .filter(|r| r.name.as_ref().map_or(false, |n| n == name))
                    .cloned()
                    .collect::<Vec<_>>();

                if matching_repos.is_empty() {
                    println!(
                        "{}",
                        ui::error(&format!("No repository with name '{}' found in config", name))
                    );
                }

                Ok(matching_repos)
            } else {
                println!(
                    "{}",
                    ui::error(&format!("No repository with name '{}' found in config", name))
                );
                Ok(vec![])
            }
        }
        // Case 3: Both provided - error
        (Some(_), Some(_)) => {
            println!("{}", ui::error("Cannot specify both --path and --name"));
            Ok(vec![])
        }
        // Case 4: Neither provided - use all active repos from config or current directory
        (None, None) => {
            if let Some(repos) = &config.repositories {
                if !repos.is_empty() {
                    Ok(repos.iter().filter(|r| r.active.unwrap_or(true)).cloned().collect())
                } else {
                    // No repos in config, use current directory
                    Ok(vec![RepositoryConfig {
                        path: std::env::current_dir()?.to_string_lossy().to_string(),
                        name: None,
                        active: Some(true),
                        branch: None,
                        include_patterns: None,
                        exclude_patterns: None,
                    }])
                }
            } else {
                // No repos in config, use current directory
                Ok(vec![RepositoryConfig {
                    path: std::env::current_dir()?.to_string_lossy().to_string(),
                    name: None,
                    active: Some(true),
                    branch: None,
                    include_patterns: None,
                    exclude_patterns: None,
                }])
            }
        }
    }
}

/// Create discovery options with smart pattern detection
fn create_discovery_options(path: &Path) -> DiscoveryOptions {
    // Dynamically detect workspace patterns
    let include_patterns = workspace_detection::detect_workspace_patterns(path);
    let exclude_patterns = workspace_detection::get_standard_exclude_patterns();

    DiscoveryOptions::new()
        .auto_detect_root(true)
        .detect_package_manager(true)
        .include_patterns(include_patterns)
        .exclude_patterns(exclude_patterns)
}

/// Output human-readable information for a repository
fn output_human_readable_for_repository(
    repo_config: &RepositoryConfig,
    verbose: bool,
    show_graph: bool,
    validate: bool,
) -> Result<()> {
    let path = PathBuf::from(&repo_config.path);

    // Display repository header
    println!(
        "{}",
        ui::section_header(&format!(
            "Repository Information {}",
            repo_config.name.as_ref().map_or("".to_string(), |n| format!("({})", n))
        ))
    );

    println!("{}", ui::key_value("Path", &repo_config.path));

    if !path.exists() {
        println!("{}", ui::error("Repository path does not exist!"));
        return Ok(());
    }

    // Create discovery options with smart pattern detection
    let options = create_discovery_options(&path);

    // Attempt to discover the workspace
    let manager = WorkspaceManager::new();
    let workspace_result = manager.discover_workspace(&path, &options);

    match workspace_result {
        Ok(workspace) => {
            let workspace = Rc::new(workspace);

            // Print Git repository information
            display_git_info(&workspace, verbose)?;

            // Print package manager information
            display_package_manager_info(&workspace)?;

            // Print package information
            display_package_info(&workspace)?;

            // Show cycle information
            display_cycle_info(&workspace)?;

            // Show graph if requested
            if show_graph {
                display_dependency_graph(&workspace)?;
            }

            // Validate if requested
            if validate {
                validate_workspace(&workspace)?;
            }

            // Detailed package information if verbose
            if verbose {
                display_detailed_package_info(&workspace)?;
            }
        }
        Err(err) => {
            display_fallback_info(&path, &err)?;
        }
    }

    Ok(())
}

// Functions for displaying different sections of information
// Each function handles a specific aspect of workspace information display

fn display_git_info(
    workspace: &Rc<sublime_monorepo_tools::Workspace>,
    verbose: bool,
) -> Result<()> {
    if let Some(repo) = workspace.git_repo() {
        println!("\n{}", ui::highlight("Git Information"));

        if let Ok(branch) = repo.get_current_branch() {
            println!("{}", ui::key_value("Current Branch", &branch));
        }

        if let Ok(sha) = repo.get_current_sha() {
            println!("{}", ui::key_value("Current Commit", &sha[0..8]));
        }

        if let Ok(status) = repo.status_porcelain() {
            println!("{}", ui::key_value("Changed Files", &status.len().to_string()));

            if !status.is_empty() && verbose {
                println!("\n{}", ui::highlight("Changed Files"));

                // Create vector of owned strings to avoid lifetime issues
                let mut status_list = Vec::new();
                for file in status.iter().take(10) {
                    status_list.push(file.clone()); // Clone to own the string
                }

                if status.len() > 10 {
                    // Create owned string for the "and more" message
                    let more_message = format!("...and {} more", status.len() - 10);
                    status_list.push(more_message);
                }

                println!("{}", ui::bullet_list(status_list));
            }
        }

        if verbose {
            if let Ok(commits) = repo.get_commits_since(None, &None) {
                println!("\n{}", ui::highlight("Recent Commits"));
                let mut commit_list = Vec::new();
                for commit in commits.iter().take(5) {
                    commit_list.push(format!(
                        "{} {} ({})",
                        &commit.hash[0..7],
                        commit.message.lines().next().unwrap_or(""),
                        commit.author_name
                    ));
                }
                println!("{}", ui::bullet_list(commit_list));
            }
        }
    }

    Ok(())
}

fn display_package_manager_info(workspace: &Rc<sublime_monorepo_tools::Workspace>) -> Result<()> {
    println!("\n{}", ui::highlight("Package Manager"));
    if let Some(pkg_manager) = workspace.package_manager() {
        println!("{}", ui::key_value("Type", &pkg_manager.to_string()));
    } else {
        println!("{}", ui::muted("No package manager detected"));
    }

    Ok(())
}

fn display_package_info(workspace: &Rc<sublime_monorepo_tools::Workspace>) -> Result<()> {
    let packages = workspace.sorted_packages();
    println!("\n{}", ui::highlight(&format!("Packages ({})", packages.len())));

    if packages.is_empty() {
        println!("{}", ui::muted("No packages found"));
    } else {
        let mut table_rows = Vec::new();

        for pkg_info in &packages {
            let info = pkg_info.borrow();
            let pkg = info.package.borrow();

            let deps = pkg.dependencies();
            let relative_path = if info.package_relative_path.is_empty() {
                ".".to_string()
            } else {
                info.package_relative_path.clone()
            };

            table_rows.push(vec![
                pkg.name().to_string(),
                pkg.version_str(),
                deps.len().to_string(),
                relative_path,
            ]);
        }

        let headers = vec![
            "Name".to_string(),
            "Version".to_string(),
            "Dependencies".to_string(),
            "Path".to_string(),
        ];
        println!("{}", ui::create_table(headers, table_rows));
    }

    Ok(())
}

fn display_cycle_info(workspace: &Rc<sublime_monorepo_tools::Workspace>) -> Result<()> {
    let cycles = workspace.get_circular_dependencies();
    if !cycles.is_empty() {
        println!("\n{}", ui::warning(&format!("Circular Dependencies ({})", cycles.len())));
        for (i, cycle) in cycles.iter().enumerate() {
            println!("  Cycle {}: {}", i + 1, ui::error_style(&cycle.join(" → ")));
        }
    }

    Ok(())
}

fn display_dependency_graph(workspace: &Rc<sublime_monorepo_tools::Workspace>) -> Result<()> {
    println!("\n{}", ui::highlight("Dependency Graph"));

    // First, get the packages from the workspace
    let packages = workspace.sorted_packages();

    // Convert package references to Package objects for graph building
    let package_objects: Vec<_> =
        packages.iter().map(|pkg_info| pkg_info.borrow().package.borrow().clone()).collect();

    // Use sublime_package_tools to build and visualize the dependency graph
    if !package_objects.is_empty() {
        // Build the dependency graph using the function from sublime_package_tools
        let graph = sublime_package_tools::build_dependency_graph_from_packages(&package_objects);

        // Check if we have cycles in the graph
        if graph.has_cycles() {
            println!("{}", ui::warning("Graph contains circular dependencies:"));
            for cycle in graph.get_cycle_strings() {
                println!("  {}", ui::error_style(&cycle.join(" → ")));
            }
            println!("\n");
        }

        // Generate ASCII visualization
        match sublime_package_tools::generate_ascii(&graph) {
            Ok(ascii) => {
                println!("{}", ascii);
            }
            Err(e) => {
                println!("{}", ui::error(&format!("Failed to generate ASCII graph: {}", e)));
            }
        }

        // If we have external dependencies, show them
        let externals = graph.find_external_dependencies();
        if !externals.is_empty() {
            println!("\n{}", ui::highlight("External Dependencies"));

            // Create vector of owned strings
            let mut external_deps = Vec::new();
            for dep in externals.iter().take(10) {
                external_deps.push(dep.clone());
            }

            if externals.len() > 10 {
                external_deps.push(format!("... and {} more", externals.len() - 10));
            }

            println!("{}", ui::bullet_list(external_deps));
        }

        // Look for version conflicts
        if let Some(conflicts) = graph.find_version_conflicts() {
            println!("\n{}", ui::warning("Version Conflicts Detected"));
            for (package, versions) in conflicts {
                println!("  {}: {}", ui::error_style(&package), versions.join(", "));
            }
        }
    } else {
        println!("{}", ui::muted("No packages found to build dependency graph"));
    }

    Ok(())
}

fn validate_workspace(workspace: &Rc<sublime_monorepo_tools::Workspace>) -> Result<()> {
    println!("\n{}", ui::highlight("Validation"));

    // Get packages for validation
    let packages = workspace.sorted_packages();

    // Convert package references to Package objects
    let package_objects: Vec<_> =
        packages.iter().map(|pkg_info| pkg_info.borrow().package.borrow().clone()).collect();

    if !package_objects.is_empty() {
        // Build the dependency graph
        let graph = sublime_package_tools::build_dependency_graph_from_packages(&package_objects);

        // Set up validation options
        let validation_options = ValidationOptions::new().treat_unresolved_as_external(true);

        // Validate the graph
        match graph.validate_with_options(&validation_options) {
            Ok(validation) => {
                if validation.has_issues() {
                    println!(
                        "{}",
                        ui::warning(&format!("Found {} issues", validation.issues().len()))
                    );
                    for issue in validation.issues() {
                        println!("  {}", ui::error_style(&format!("{issue:?}")));
                    }
                } else {
                    println!("{}", ui::success("No issues found"));
                }
            }
            Err(e) => {
                println!("{}", ui::error(&format!("Validation failed: {}", e)));
            }
        }
    } else {
        println!("{}", ui::muted("No packages found to validate"));
    }

    Ok(())
}

fn display_detailed_package_info(workspace: &Rc<sublime_monorepo_tools::Workspace>) -> Result<()> {
    let packages = workspace.sorted_packages();

    for pkg_info in &packages {
        let info = pkg_info.borrow();
        let pkg = info.package.borrow();

        println!("\n{}", ui::highlight(&format!("Package: {}", pkg.name())));
        println!("{}", ui::key_value("Version", &pkg.version_str()));
        println!("{}", ui::key_value("Path", &info.package_path));

        let deps = pkg.dependencies();
        if !deps.is_empty() {
            println!("\n  {}", ui::secondary_style("Dependencies:"));
            for dep in deps {
                let dep = dep.borrow();
                println!("    {} {}", dep.name(), dep.version());
            }
        }

        // Show packages that depend on this one
        let dependents = workspace.dependents_of(pkg.name());
        if !dependents.is_empty() {
            println!("\n  {}", ui::secondary_style("Dependents:"));
            for dep_pkg in dependents {
                let dep = dep_pkg.borrow();
                println!("    {}", dep.package.borrow().name());
            }
        }

        // Check if this package is part of a cycle
        if workspace.is_in_cycle(pkg.name()) {
            if let Some(cycle) = workspace.get_cycle_for_package(pkg.name()) {
                println!("\n  {}", ui::warning_style("Part of dependency cycle:"));
                println!("    {}", ui::error_style(&cycle.join(" → ")));
            }
        }
    }

    Ok(())
}

fn display_fallback_info(path: &Path, err: &sublime_monorepo_tools::WorkspaceError) -> Result<()> {
    // Fallback to basic information if workspace discovery fails
    println!("{}", ui::warning(&format!("Failed to discover workspace: {}", err)));

    // Try to get some package information using standard tools
    if let Some(package_manager) = sublime_standard_tools::detect_package_manager(path) {
        println!("\n{}", ui::highlight("Package Manager"));
        println!("{}", ui::key_value("Type", &package_manager.to_string()));
    }

    // Try to get Git information
    if let Ok(repo) = Repo::open(path.to_string_lossy().as_ref()) {
        println!("\n{}", ui::highlight("Git Information"));

        if let Ok(branch) = repo.get_current_branch() {
            println!("{}", ui::key_value("Current Branch", &branch));
        }

        if let Ok(status) = repo.status_porcelain() {
            println!("{}", ui::key_value("Changed Files", &status.len().to_string()));
        }
    }

    println!(
        "\n{}",
        ui::info(
            "You might be able to get more information by specifying the correct repository path."
        )
    );

    Ok(())
}

/// Output repository information in JSON format
fn output_json_for_repositories(
    repositories: &[RepositoryConfig],
    verbose: bool,
    show_graph: bool,
    show_cycles: bool,
    validate: bool,
) -> Result<()> {
    let mut repos_data = Vec::new();

    for repo_config in repositories {
        let path = PathBuf::from(&repo_config.path);
        log::info!("Analyzing repository at {}", path.display());

        if !path.exists() {
            repos_data.push(serde_json::json!({
                "error": "Repository path does not exist",
                "path": repo_config.path,
                "name": repo_config.name,
            }));
            continue;
        }

        // Create discovery options with smart pattern detection
        let options = create_discovery_options(&path);

        // Discover workspace
        let manager = WorkspaceManager::new();

        match manager.discover_workspace(&path, &options) {
            Ok(workspace) => {
                let repo_data = generate_repo_json_data(
                    &workspace,
                    repo_config,
                    verbose,
                    show_graph,
                    show_cycles,
                    validate,
                )?;

                repos_data.push(repo_data);
            }
            Err(err) => {
                // Generate fallback JSON with basic info
                let repo_data = generate_fallback_json_data(&path, repo_config, &err)?;
                repos_data.push(repo_data);
            }
        }
    }

    // Output the final JSON
    let result = if repositories.len() == 1 {
        // Just output the single repo data
        repos_data[0].clone()
    } else {
        // Wrap in an array for multiple repos
        serde_json::Value::Array(repos_data)
    };

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

/// Generate JSON data for a successfully discovered workspace
fn generate_repo_json_data(
    workspace: &sublime_monorepo_tools::Workspace,
    repo_config: &RepositoryConfig,
    verbose: bool,
    show_graph: bool,
    show_cycles: bool,
    validate: bool,
) -> Result<serde_json::Value> {
    let mut repo_data = serde_json::Map::new();

    // Add basic repo info
    repo_data.insert("path".into(), serde_json::Value::String(repo_config.path.clone()));
    if let Some(name) = &repo_config.name {
        repo_data.insert("name".into(), serde_json::Value::String(name.clone()));
    }

    // Package manager info
    if let Some(pkg_manager) = workspace.package_manager() {
        repo_data
            .insert("packageManager".into(), serde_json::Value::String(pkg_manager.to_string()));
    }

    // Git info if available
    if let Some(repo) = workspace.git_repo() {
        let mut git_info = serde_json::Map::new();

        if let Ok(branch) = repo.get_current_branch() {
            git_info.insert("branch".into(), serde_json::Value::String(branch));
        }

        if let Ok(sha) = repo.get_current_sha() {
            git_info.insert("commit".into(), serde_json::Value::String(sha));
        }

        // Add git status information
        if let Ok(status) = repo.status_porcelain() {
            git_info.insert("changedFiles".into(), serde_json::Value::Number(status.len().into()));
        }

        repo_data.insert("git".into(), serde_json::Value::Object(git_info));
    }

    // Package information
    let packages = workspace.sorted_packages();
    let mut package_list = Vec::new();

    for pkg_info in &packages {
        let info = pkg_info.borrow();
        let pkg = info.package.borrow();

        let mut pkg_data = serde_json::Map::new();
        pkg_data.insert("name".into(), serde_json::Value::String(pkg.name().to_string()));
        pkg_data.insert("version".into(), serde_json::Value::String(pkg.version_str()));
        pkg_data.insert("path".into(), serde_json::Value::String(info.package_path.clone()));

        // Add dependency information if verbose
        if verbose {
            let deps = pkg.dependencies();
            let mut dep_list = serde_json::Map::new();

            for dep in deps {
                let dep = dep.borrow();
                dep_list.insert(
                    dep.name().to_string(),
                    serde_json::Value::String(dep.version().to_string()),
                );
            }

            pkg_data.insert("dependencies".into(), serde_json::Value::Object(dep_list));

            // Get dependents of this package
            let dependents = workspace.dependents_of(pkg.name());
            let dependent_names: Vec<String> = dependents
                .iter()
                .map(|dep| dep.borrow().package.borrow().name().to_string())
                .collect();

            pkg_data.insert("dependents".into(), serde_json::json!(dependent_names));
        }

        package_list.push(serde_json::Value::Object(pkg_data));
    }

    repo_data.insert("packages".into(), serde_json::Value::Array(package_list));

    // Analyze dependency cycles if requested
    if show_cycles || show_graph {
        add_cycle_data_to_json(workspace, &mut repo_data)?;
    }

    // Validate the workspace if requested
    if validate {
        add_validation_data_to_json(workspace, &mut repo_data)?;
    }

    Ok(serde_json::Value::Object(repo_data))
}

/// Add cycle detection data to JSON output
fn add_cycle_data_to_json(
    workspace: &sublime_monorepo_tools::Workspace,
    repo_data: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<()> {
    // Get the packages for graph analysis
    let packages = workspace.sorted_packages();
    let package_objects: Vec<_> =
        packages.iter().map(|pkg_info| pkg_info.borrow().package.borrow().clone()).collect();

    if !package_objects.is_empty() {
        // Build the dependency graph
        let graph = sublime_package_tools::build_dependency_graph_from_packages(&package_objects);

        // Check for cycles
        if graph.has_cycles() {
            repo_data.insert("cycles".into(), serde_json::json!(graph.get_cycle_strings()));
        }
    }

    Ok(())
}

/// Add validation data to JSON output
fn add_validation_data_to_json(
    workspace: &sublime_monorepo_tools::Workspace,
    repo_data: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<()> {
    // Get the packages for validation
    let packages = workspace.sorted_packages();
    let package_objects: Vec<_> =
        packages.iter().map(|pkg_info| pkg_info.borrow().package.borrow().clone()).collect();

    if !package_objects.is_empty() {
        // Build the dependency graph
        let graph = sublime_package_tools::build_dependency_graph_from_packages(&package_objects);

        // Set up validation options
        let validation_options = ValidationOptions::new().treat_unresolved_as_external(true);

        // Validate the graph
        if let Ok(validation) = graph.validate_with_options(&validation_options) {
            let mut validation_result = serde_json::Map::new();

            validation_result
                .insert("hasIssues".into(), serde_json::Value::Bool(validation.has_issues()));

            let issues: Vec<String> =
                validation.issues().iter().map(|issue| format!("{issue:?}")).collect();

            validation_result.insert("issues".into(), serde_json::json!(issues));

            repo_data.insert("validation".into(), serde_json::Value::Object(validation_result));
        }
    }

    Ok(())
}

/// Generate fallback JSON data when workspace discovery fails
fn generate_fallback_json_data(
    path: &Path,
    repo_config: &RepositoryConfig,
    err: &sublime_monorepo_tools::WorkspaceError,
) -> Result<serde_json::Value> {
    let mut repo_data = serde_json::Map::new();

    // Basic info
    repo_data.insert("path".into(), serde_json::Value::String(repo_config.path.clone()));
    if let Some(name) = &repo_config.name {
        repo_data.insert("name".into(), serde_json::Value::String(name.clone()));
    }

    // Error info
    repo_data.insert(
        "error".into(),
        serde_json::Value::String(format!("Failed to discover workspace: {}", err)),
    );

    // Try to detect package manager with standard tools
    if let Some(package_manager) = sublime_standard_tools::detect_package_manager(path) {
        repo_data.insert(
            "packageManager".into(),
            serde_json::Value::String(package_manager.to_string()),
        );
    }

    // Try to get Git information manually
    if let Ok(repo) = Repo::open(path.to_string_lossy().as_ref()) {
        let mut git_info = serde_json::Map::new();

        if let Ok(branch) = repo.get_current_branch() {
            git_info.insert("branch".into(), serde_json::Value::String(branch));
        }

        repo_data.insert("git".into(), serde_json::Value::Object(git_info));
    }

    Ok(serde_json::Value::Object(repo_data))
}
