//! # Cache and Configuration Example
//!
//! This example demonstrates how to use the cache and configuration
//! functionality to store and retrieve data efficiently, manage
//! application settings, and handle various data formats.

#![allow(clippy::print_stdout)]
#![allow(clippy::uninlined_format_args)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use std::{collections::HashMap, path::PathBuf, thread, time::Duration};
use sublime_standard_tools::{
    cache::{Cache, CacheConfig, CacheStrategy},
    config::{ConfigManager, ConfigScope, ConfigValue},
    error::StandardResult,
};

/// Demonstrates the use of the cache system
fn cache_example() {
    println!("=== Cache Example ===");

    // Create a cache with custom configuration
    let config = CacheConfig {
        default_ttl: Duration::from_secs(10),
        capacity: 100,
        strategy: CacheStrategy::LRU,
    };

    println!("Creating cache with TTL: 10s, Capacity: 100, Strategy: LRU");
    let cache = Cache::<String, String>::with_config(config);

    // Store some values
    cache.put("key1".to_string(), "value1".to_string());
    cache.put("key2".to_string(), "value2".to_string());
    cache.put_with_ttl(
        "short_expiry".to_string(),
        "expires quickly".to_string(),
        Duration::from_secs(2),
    );

    println!("Stored 3 values in cache");
    println!("Cache now contains {} items", cache.len());

    // Retrieve values
    if let Some(value) = cache.get(&"key1".to_string()) {
        println!("Retrieved value for key1: {}", value);
    }

    if let Some(value) = cache.get(&"key2".to_string()) {
        println!("Retrieved value for key2: {}", value);
    }

    // Try a non-existent key
    if let Some(value) = cache.get(&"nonexistent".to_string()) {
        println!("Retrieved value for nonexistent: {}", value);
    } else {
        println!("Key 'nonexistent' not found in cache as expected");
    }

    // Test cache statistics
    println!("\nCache Statistics:");
    println!("  Items: {}", cache.len());
    println!("  Hits: {}", cache.hits());
    println!("  Misses: {}", cache.misses());
    println!("  Hit rate: {:.2}%", cache.hit_rate() * 100.0);

    // Test expiration
    println!("\nWaiting for 'short_expiry' key to expire...");
    thread::sleep(Duration::from_secs(3));

    if let Some(value) = cache.get(&"short_expiry".to_string()) {
        println!("Retrieved value for short_expiry: {}", value);
    } else {
        println!("Key 'short_expiry' expired as expected");
    }

    // Clean expired entries and check statistics again
    cache.clean();
    println!("\nAfter cleaning expired entries:");
    println!("  Items: {}", cache.len());
    println!("  Hits: {}", cache.hits());
    println!("  Misses: {}", cache.misses());
    println!("  Hit rate: {:.2}%", cache.hit_rate() * 100.0);
}

/// Demonstrates the use of the configuration system
fn config_example() -> StandardResult<()> {
    println!("\n=== Configuration Example ===");

    // Create a temporary directory for configuration files
    let temp_dir = tempfile::tempdir().map_err(|e| {
        sublime_standard_tools::error::StandardError::operation(format!(
            "Failed to create temporary directory: {}",
            e
        ))
    })?;

    println!("Using temporary directory for config files: {}", temp_dir.path().display());

    // Create a configuration manager
    let mut config_manager = ConfigManager::new();

    // Set paths for different scopes
    let global_config_path = temp_dir.path().join("global.json");
    let user_config_path = temp_dir.path().join("user.json");
    let project_config_path = temp_dir.path().join("project.json");

    config_manager.set_path(ConfigScope::Global, &global_config_path);
    config_manager.set_path(ConfigScope::User, &user_config_path);
    config_manager.set_path(ConfigScope::Project, &project_config_path);

    println!("Setting configuration values...");

    // Set various configuration values
    config_manager.set("app.name", ConfigValue::String("My Node.js App".to_string()));
    config_manager.set("app.version", ConfigValue::String("1.0.0".to_string()));
    config_manager.set("server.port", ConfigValue::Integer(3000));
    config_manager.set("server.host", ConfigValue::String("localhost".to_string()));
    config_manager.set("debug", ConfigValue::Boolean(true));
    config_manager.set("timeout", ConfigValue::Integer(30));

    // Create a nested structure
    let mut database = HashMap::new();
    database.insert("host".to_string(), ConfigValue::String("localhost".to_string()));
    database.insert("port".to_string(), ConfigValue::Integer(5432));
    database.insert("username".to_string(), ConfigValue::String("admin".to_string()));
    database.insert("password".to_string(), ConfigValue::String("password123".to_string()));
    config_manager.set("database", ConfigValue::Map(database));

    // Create an array
    let allowed_origins = vec![
        ConfigValue::String("http://localhost:3000".to_string()),
        ConfigValue::String("https://example.com".to_string()),
    ];
    config_manager.set("server.cors.allowedOrigins", ConfigValue::Array(allowed_origins));

    // Save configurations
    println!("Saving configurations to disk...");
    config_manager.save_all()?;

    println!("Configuration saved successfully");

    // Create a new config manager and load the saved configurations
    println!("\nCreating new configuration manager and loading saved config...");

    let mut new_config_manager = ConfigManager::new();
    new_config_manager.set_path(ConfigScope::Global, &global_config_path);
    new_config_manager.set_path(ConfigScope::User, &user_config_path);
    new_config_manager.set_path(ConfigScope::Project, &project_config_path);

    new_config_manager.load_all()?;

    // Read and display configuration values
    println!("\nLoaded Configuration Values:");

    if let Some(app_name) = new_config_manager.get("app.name") {
        if let Some(name) = app_name.as_string() {
            println!("  app.name: {}", name);
        }
    }

    if let Some(port) = new_config_manager.get("server.port") {
        if let Some(port_num) = port.as_integer() {
            println!("  server.port: {}", port_num);
        }
    }

    if let Some(debug) = new_config_manager.get("debug") {
        if let Some(enabled) = debug.as_boolean() {
            println!("  debug: {}", enabled);
        }
    }

    if let Some(database_config) = new_config_manager.get("database") {
        if let Some(db_map) = database_config.as_map() {
            println!("  Database Configuration:");
            for (key, value) in db_map {
                match value {
                    ConfigValue::String(s) => println!("    {}: {}", key, s),
                    ConfigValue::Integer(i) => println!("    {}: {}", key, i),
                    _ => println!("    {}: <complex value>", key),
                }
            }
        }
    }

    if let Some(origins) = new_config_manager.get("server.cors.allowedOrigins") {
        if let Some(origins_array) = origins.as_array() {
            println!("  Allowed Origins:");
            for (i, origin) in origins_array.iter().enumerate() {
                if let Some(origin_str) = origin.as_string() {
                    println!("    {}. {}", i + 1, origin_str);
                }
            }
        }
    }

    // Modify configuration and save again
    println!("\nModifying configuration...");
    new_config_manager.set("app.version", ConfigValue::String("1.1.0".to_string()));
    new_config_manager.set("server.port", ConfigValue::Integer(4000));
    new_config_manager.set("debug", ConfigValue::Boolean(false));

    // Save only the Project scope
    new_config_manager.save_to_file(&project_config_path)?;

    println!("Configuration modified and saved successfully");

    Ok(())
}

/// Demonstrates a real-world use case combining cache and configuration
#[allow(clippy::cast_sign_loss)]
fn combined_example() {
    println!("\n=== Combined Cache and Configuration Example ===");
    println!("Simulating a Node.js project analyzer that caches results");

    // Setup configuration
    let config_manager = ConfigManager::new();

    // Configure analysis settings
    let mut analysis_config = HashMap::new();
    analysis_config.insert("maxDepth".to_string(), ConfigValue::Integer(3));
    analysis_config.insert("includeDependencies".to_string(), ConfigValue::Boolean(true));
    analysis_config.insert("cacheResults".to_string(), ConfigValue::Boolean(true));
    analysis_config.insert(
        "cacheTTL".to_string(),
        ConfigValue::Integer(3600), // 1 hour in seconds
    );

    config_manager.set("analysis", ConfigValue::Map(analysis_config));

    // Setup cache based on configuration
    let cache_ttl = if let Some(analysis) = config_manager.get("analysis") {
        if let Some(map) = analysis.as_map() {
            if let Some(ttl) = map.get("cacheTTL") {
                if let Some(seconds) = ttl.as_integer() {
                    Duration::from_secs(seconds as u64)
                } else {
                    Duration::from_secs(3600) // Default 1 hour
                }
            } else {
                Duration::from_secs(3600) // Default 1 hour
            }
        } else {
            Duration::from_secs(3600) // Default 1 hour
        }
    } else {
        Duration::from_secs(3600) // Default 1 hour
    };

    println!("Creating project analyzer with cache TTL: {:?}", cache_ttl);

    let cache_config =
        CacheConfig { default_ttl: cache_ttl, capacity: 50, strategy: CacheStrategy::LRU };

    let project_cache = Cache::<PathBuf, ProjectAnalysis>::with_config(cache_config);

    // Simulate analyzing some projects
    let projects = [
        PathBuf::from("/path/to/project1"),
        PathBuf::from("/path/to/project2"),
        PathBuf::from("/path/to/project3"),
    ];

    println!("Analyzing projects...");

    for (i, project_path) in projects.iter().enumerate() {
        println!("\nAnalyzing project at: {}", project_path.display());

        // Check if we have a cached result
        if let Some(analysis) = project_cache.get(project_path) {
            println!("  Found cached analysis result:");
            println!("  - Dependencies: {}", analysis.dependencies.len());
            println!("  - Dev Dependencies: {}", analysis.dev_dependencies.len());
            println!("  - Scripts: {}", analysis.scripts.len());
            println!("  - Analysis date: {}", analysis.date);
        } else {
            // Simulate a new analysis
            println!("  Performing new analysis (simulated)...");
            thread::sleep(Duration::from_millis(500)); // Simulate work

            // Create a simulated analysis result
            let analysis = ProjectAnalysis {
                name: format!("project{}", i + 1),
                version: "1.0.0".to_string(),
                dependencies: vec![
                    format!("dep{}-1", i + 1),
                    format!("dep{}-2", i + 1),
                    format!("dep{}-3", i + 1),
                ],
                dev_dependencies: vec![
                    format!("dev-dep{}-1", i + 1),
                    format!("dev-dep{}-2", i + 1),
                ],
                scripts: vec![format!("script{}-1", i + 1), format!("script{}-2", i + 1)],
                date: chrono::Local::now().to_rfc3339(),
            };

            // Cache the result
            project_cache.put(project_path.clone(), analysis.clone());

            println!("  Analysis complete and cached:");
            println!("  - Dependencies: {}", analysis.dependencies.len());
            println!("  - Dev Dependencies: {}", analysis.dev_dependencies.len());
            println!("  - Scripts: {}", analysis.scripts.len());
            println!("  - Analysis date: {}", analysis.date);
        }
    }

    // Print cache statistics
    println!("\nCache Statistics:");
    println!("  Items: {}", project_cache.len());
    println!("  Hits: {}", project_cache.hits());
    println!("  Misses: {}", project_cache.misses());
    println!("  Hit rate: {:.2}%", project_cache.hit_rate() * 100.0);
}

/// Represents the analysis result of a project
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ProjectAnalysis {
    name: String,
    version: String,
    dependencies: Vec<String>,
    dev_dependencies: Vec<String>,
    scripts: Vec<String>,
    date: String, // ISO formatted date string
}

fn main() -> StandardResult<()> {
    // Run the examples
    cache_example();
    config_example()?;
    combined_example();

    Ok(())
}
