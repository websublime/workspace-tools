//! Example demonstrating Package Tools configuration integration.
//!
//! This example shows how to use the new PackageToolsConfig that integrates
//! with StandardConfig to provide comprehensive configuration management.

use sublime_package_tools::config::{
    PackageToolsConfig, VersionBumpStrategy, CircularDependencyHandling,
    DependencyProtocol, ProjectContextType, MemoryOptimizationLevel,
};
use sublime_standard_tools::config::traits::Configurable;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Package Tools Configuration Integration Example");
    println!("==================================================");

    // Create default configuration
    let mut config = PackageToolsConfig::default();
    println!("✅ Created default configuration");

    // Display default values
    println!("\n📋 Default Configuration Values:");
    println!("  Version bump strategy: {:?}", config.version_bumping.default_strategy);
    println!("  Max concurrent downloads: {}", config.dependency_resolution.max_concurrent_downloads);
    println!("  Circular dependency handling: {:?}", config.circular_dependency_handling.handling_strategy);
    println!("  Auto-detect context: {}", config.context_aware.auto_detect_context);
    println!("  Cache enabled: {}", config.cache.enable_cache);

    // Validate the configuration
    config.validate()?;
    println!("✅ Configuration validation passed");

    // Create a custom configuration
    let mut custom_config = PackageToolsConfig {
        version_bumping: sublime_package_tools::config::VersionBumpConfig {
            default_strategy: VersionBumpStrategy::Minor,
            enable_cascade_bumping: true,
            snapshot_prefix: "dev".to_string(),
            ..Default::default()
        },
        dependency_resolution: sublime_package_tools::config::ResolutionConfig {
            max_concurrent_downloads: 5,
            supported_protocols: vec![
                DependencyProtocol::Npm,
                DependencyProtocol::Git,
                DependencyProtocol::Workspace,
            ],
            ..Default::default()
        },
        circular_dependency_handling: sublime_package_tools::config::CircularDependencyConfig {
            handling_strategy: CircularDependencyHandling::Warn,
            allow_dev_cycles: true,
            max_cycle_depth: 5,
            ..Default::default()
        },
        context_aware: sublime_package_tools::config::ContextAwareConfig {
            force_context: Some(ProjectContextType::Monorepo),
            ..Default::default()
        },
        performance: sublime_package_tools::config::PerformanceConfig {
            memory_optimization: MemoryOptimizationLevel::Aggressive,
            max_worker_threads: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    println!("\n🛠️  Custom Configuration Values:");
    println!("  Version bump strategy: {:?}", custom_config.version_bumping.default_strategy);
    println!("  Max concurrent downloads: {}", custom_config.dependency_resolution.max_concurrent_downloads);
    println!("  Cascade bumping enabled: {}", custom_config.version_bumping.enable_cascade_bumping);
    println!("  Forced context: {:?}", custom_config.context_aware.force_context);
    println!("  Memory optimization: {:?}", custom_config.performance.memory_optimization);

    // Validate custom configuration
    custom_config.validate()?;
    println!("✅ Custom configuration validation passed");

    // Demonstrate configuration merging
    println!("\n🔄 Demonstrating configuration merging...");
    config.merge_with(custom_config)?;
    
    println!("✅ Configuration merge completed");
    println!("  Updated version bump strategy: {:?}", config.version_bumping.default_strategy);
    println!("  Updated max concurrent downloads: {}", config.dependency_resolution.max_concurrent_downloads);
    println!("  Updated cascade bumping: {}", config.version_bumping.enable_cascade_bumping);

    // Show environment variable support
    println!("\n🌍 Environment Variable Support:");
    println!("  Set SUBLIME_PKG_VERSION_BUMP_STRATEGY=major to override default");
    println!("  Set SUBLIME_PKG_CONCURRENT_DOWNLOADS=15 to override downloads");
    println!("  Set SUBLIME_PKG_CIRCULAR_DEP_HANDLING=error to change handling");
    println!("  See config documentation for full list of variables");

    // Final validation after merge
    config.validate()?;
    println!("\n✅ Final validation passed - configuration is ready for use!");

    Ok(())
}