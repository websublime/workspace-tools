//! Package tools configuration integration with StandardConfig.
//!
//! This module defines the package tools configuration structure that extends
//! StandardConfig with package-specific options. It provides comprehensive
//! configuration for all package-related operations including version bumping,
//! dependency resolution, monorepo settings, and circular dependency handling.
//!
//! ## Environment Variable Overrides
//!
//! Package-specific configuration can be overridden using environment variables:
//!
//! ### Version Bumping Configuration
//! - `SUBLIME_PKG_VERSION_BUMP_STRATEGY`: Version bump strategy (major,minor,patch,snapshot,cascade)
//! - `SUBLIME_PKG_CASCADE_BUMPING`: Enable cascade bumping (true/false)
//! - `SUBLIME_PKG_SNAPSHOT_PREFIX`: Snapshot version prefix (default: "snapshot")
//! - `SUBLIME_PKG_AFFECTED_DETECTION`: Affected package detection strategy (auto,manual,git-diff)
//!
//! ### Dependency Resolution Configuration
//! - `SUBLIME_PKG_CONCURRENT_DOWNLOADS`: Max concurrent downloads (1-50)
//! - `SUBLIME_PKG_REGISTRY_TIMEOUT`: Registry request timeout in seconds (1-300)
//! - `SUBLIME_PKG_CACHE_TTL`: Cache time-to-live in seconds (60-86400)
//! - `SUBLIME_PKG_SUPPORTED_PROTOCOLS`: Comma-separated supported protocols (npm,jsr,git,file,workspace,url)
//! - `SUBLIME_PKG_ENABLE_WORKSPACE_PROTOCOLS`: Enable workspace: protocols (true/false)
//!
//! ### Circular Dependency Configuration
//! - `SUBLIME_PKG_CIRCULAR_DEP_HANDLING`: How to handle circular deps (warn,error,ignore)
//! - `SUBLIME_PKG_ALLOW_DEV_CYCLES`: Allow cycles in dev dependencies (true/false)
//! - `SUBLIME_PKG_ALLOW_OPTIONAL_CYCLES`: Allow cycles in optional dependencies (true/false)
//! - `SUBLIME_PKG_MAX_CYCLE_DEPTH`: Maximum allowed cycle depth (1-20)
//!
//! All environment variables are validated with reasonable bounds and will fall back
//! to hardcoded defaults if invalid values are provided.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use sublime_standard_tools::config::traits::Configurable;
use sublime_standard_tools::error::ConfigResult;

/// The package tools configuration for sublime-package-tools.
///
/// This configuration extends StandardConfig functionality with package-specific
/// options for version management, dependency resolution, and monorepo operations.
/// It integrates seamlessly with the standard configuration system.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::config::PackageToolsConfig;
///
/// let config = PackageToolsConfig::default();
/// assert_eq!(config.version_bumping.default_strategy, VersionBumpStrategy::Patch);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageToolsConfig {
    /// Configuration version for migration support
    #[serde(default = "default_config_version")]
    pub version: String,

    /// Version bumping configuration
    #[serde(default)]
    pub version_bumping: VersionBumpConfig,

    /// Dependency resolution configuration
    #[serde(default)]
    pub dependency_resolution: ResolutionConfig,

    /// Circular dependency handling configuration
    #[serde(default)]
    pub circular_dependency_handling: CircularDependencyConfig,

    /// Context-aware features configuration
    #[serde(default)]
    pub context_aware: ContextAwareConfig,

    /// Performance optimization configuration
    #[serde(default)]
    pub performance: PerformanceConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,
}

impl Default for PackageToolsConfig {
    fn default() -> Self {
        Self {
            version: default_config_version(),
            version_bumping: VersionBumpConfig::default(),
            dependency_resolution: ResolutionConfig::default(),
            circular_dependency_handling: CircularDependencyConfig::default(),
            context_aware: ContextAwareConfig::default(),
            performance: PerformanceConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl Configurable for PackageToolsConfig {
    fn validate(&self) -> ConfigResult<()> {
        // Validate version bumping configuration
        if self.version_bumping.snapshot_prefix.is_empty() {
            return Err("Snapshot prefix cannot be empty".into());
        }

        // Validate dependency resolution configuration
        if self.dependency_resolution.max_concurrent_downloads == 0 {
            return Err("Max concurrent downloads must be greater than 0".into());
        }

        if self.dependency_resolution.max_concurrent_downloads > 50 {
            return Err("Max concurrent downloads cannot exceed 50".into());
        }

        if self.dependency_resolution.registry_timeout.as_secs() == 0 {
            return Err("Registry timeout must be greater than 0".into());
        }

        if self.dependency_resolution.registry_timeout.as_secs() > 300 {
            return Err("Registry timeout cannot exceed 300 seconds".into());
        }

        // Validate circular dependency configuration
        if self.circular_dependency_handling.max_cycle_depth == 0 {
            return Err("Max cycle depth must be greater than 0".into());
        }

        if self.circular_dependency_handling.max_cycle_depth > 20 {
            return Err("Max cycle depth cannot exceed 20".into());
        }

        // Validate performance configuration
        if self.performance.max_worker_threads == 0 {
            return Err("Max worker threads must be greater than 0".into());
        }

        if self.performance.batch_processing_size == 0 {
            return Err("Batch processing size must be greater than 0".into());
        }

        // Validate cache configuration
        if self.cache.cache_ttl.as_secs() < 60 {
            return Err("Cache TTL must be at least 60 seconds".into());
        }

        if self.cache.max_cache_size_mb == 0 {
            return Err("Max cache size must be greater than 0".into());
        }

        if self.cache.compression_level > 9 {
            return Err("Cache compression level cannot exceed 9".into());
        }

        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        // Version is always taken from other if different
        if self.version != other.version {
            self.version = other.version;
        }

        // Merge sub-configurations
        self.version_bumping.merge_with(other.version_bumping)
            .map_err(|e| format!("Failed to merge version bumping config: {e}"))?;
        
        self.dependency_resolution.merge_with(other.dependency_resolution)
            .map_err(|e| format!("Failed to merge dependency resolution config: {e}"))?;
        
        self.circular_dependency_handling.merge_with(other.circular_dependency_handling)
            .map_err(|e| format!("Failed to merge circular dependency config: {e}"))?;
        
        self.context_aware.merge_with(other.context_aware)
            .map_err(|e| format!("Failed to merge context aware config: {e}"))?;
        
        self.performance.merge_with(other.performance)
            .map_err(|e| format!("Failed to merge performance config: {e}"))?;
        
        self.cache.merge_with(other.cache)
            .map_err(|e| format!("Failed to merge cache config: {e}"))?;

        Ok(())
    }
}

/// Configuration for version bumping operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBumpConfig {
    /// Default version bump strategy
    #[serde(default = "default_version_bump_strategy")]
    pub default_strategy: VersionBumpStrategy,

    /// Enable cascade bumping for monorepos
    #[serde(default = "default_cascade_bumping")]
    pub enable_cascade_bumping: bool,

    /// Prefix for snapshot versions
    #[serde(default = "default_snapshot_prefix")]
    pub snapshot_prefix: String,

    /// How to detect affected packages
    #[serde(default = "default_affected_detection")]
    pub affected_detection: AffectedDetectionStrategy,

    /// Enable SHA appending for snapshot versions
    #[serde(default = "default_true")]
    pub append_sha_to_snapshots: bool,

    /// Custom version patterns for different package types
    #[serde(default)]
    pub custom_version_patterns: HashMap<String, String>,
}

impl Default for VersionBumpConfig {
    fn default() -> Self {
        Self {
            default_strategy: default_version_bump_strategy(),
            enable_cascade_bumping: default_cascade_bumping(),
            snapshot_prefix: default_snapshot_prefix(),
            affected_detection: default_affected_detection(),
            append_sha_to_snapshots: true,
            custom_version_patterns: HashMap::new(),
        }
    }
}

impl VersionBumpConfig {
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.default_strategy = other.default_strategy;
        self.enable_cascade_bumping = other.enable_cascade_bumping;
        if !other.snapshot_prefix.is_empty() {
            self.snapshot_prefix = other.snapshot_prefix;
        }
        self.affected_detection = other.affected_detection;
        self.append_sha_to_snapshots = other.append_sha_to_snapshots;
        self.custom_version_patterns.extend(other.custom_version_patterns);
        Ok(())
    }
}

/// Configuration for dependency resolution operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionConfig {
    /// Maximum concurrent downloads
    #[serde(default = "default_concurrent_downloads")]
    pub max_concurrent_downloads: usize,

    /// Registry request timeout
    #[serde(default = "default_registry_timeout", with = "humantime_serde")]
    pub registry_timeout: Duration,

    /// Supported dependency protocols
    #[serde(default = "default_supported_protocols")]
    pub supported_protocols: Vec<DependencyProtocol>,

    /// Enable workspace: protocol support
    #[serde(default = "default_true")]
    pub enable_workspace_protocols: bool,

    /// Fallback registries for resolution
    #[serde(default = "default_fallback_registries")]
    pub fallback_registries: Vec<String>,

    /// Custom protocol handlers
    #[serde(default)]
    pub custom_protocol_handlers: HashMap<String, String>,

    /// Enable protocol validation
    #[serde(default = "default_true")]
    pub validate_protocols: bool,
}

impl Default for ResolutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: default_concurrent_downloads(),
            registry_timeout: default_registry_timeout(),
            supported_protocols: default_supported_protocols(),
            enable_workspace_protocols: true,
            fallback_registries: default_fallback_registries(),
            custom_protocol_handlers: HashMap::new(),
            validate_protocols: true,
        }
    }
}

impl ResolutionConfig {
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        if other.max_concurrent_downloads > 0 {
            self.max_concurrent_downloads = other.max_concurrent_downloads;
        }
        if other.registry_timeout.as_secs() > 0 {
            self.registry_timeout = other.registry_timeout;
        }
        if !other.supported_protocols.is_empty() {
            self.supported_protocols = other.supported_protocols;
        }
        self.enable_workspace_protocols = other.enable_workspace_protocols;
        if !other.fallback_registries.is_empty() {
            self.fallback_registries = other.fallback_registries;
        }
        self.custom_protocol_handlers.extend(other.custom_protocol_handlers);
        self.validate_protocols = other.validate_protocols;
        Ok(())
    }
}

/// Configuration for circular dependency handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependencyConfig {
    /// How to handle circular dependencies
    #[serde(default = "default_circular_handling")]
    pub handling_strategy: CircularDependencyHandling,

    /// Allow cycles in development dependencies
    #[serde(default = "default_true")]
    pub allow_dev_cycles: bool,

    /// Allow cycles in optional dependencies
    #[serde(default = "default_true")]
    pub allow_optional_cycles: bool,

    /// Maximum allowed cycle depth
    #[serde(default = "default_max_cycle_depth")]
    pub max_cycle_depth: usize,

    /// Enable cycle detection warnings
    #[serde(default = "default_true")]
    pub enable_warnings: bool,

    /// Custom cycle detection rules
    #[serde(default)]
    pub custom_rules: HashMap<String, bool>,
}

impl Default for CircularDependencyConfig {
    fn default() -> Self {
        Self {
            handling_strategy: default_circular_handling(),
            allow_dev_cycles: true,
            allow_optional_cycles: true,
            max_cycle_depth: default_max_cycle_depth(),
            enable_warnings: true,
            custom_rules: HashMap::new(),
        }
    }
}

impl CircularDependencyConfig {
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.handling_strategy = other.handling_strategy;
        self.allow_dev_cycles = other.allow_dev_cycles;
        self.allow_optional_cycles = other.allow_optional_cycles;
        if other.max_cycle_depth > 0 {
            self.max_cycle_depth = other.max_cycle_depth;
        }
        self.enable_warnings = other.enable_warnings;
        self.custom_rules.extend(other.custom_rules);
        Ok(())
    }
}

/// Configuration for context-aware features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareConfig {
    /// Enable automatic context detection
    #[serde(default = "default_true")]
    pub auto_detect_context: bool,

    /// Force specific context type
    #[serde(default)]
    pub force_context: Option<ProjectContextType>,

    /// Enable single repository optimizations
    #[serde(default = "default_true")]
    pub enable_single_repo_optimizations: bool,

    /// Enable monorepo-specific features
    #[serde(default = "default_true")]
    pub enable_monorepo_features: bool,

    /// Context detection cache duration
    #[serde(default = "default_context_cache_duration", with = "humantime_serde")]
    pub context_cache_duration: Duration,
}

impl Default for ContextAwareConfig {
    fn default() -> Self {
        Self {
            auto_detect_context: true,
            force_context: None,
            enable_single_repo_optimizations: true,
            enable_monorepo_features: true,
            context_cache_duration: default_context_cache_duration(),
        }
    }
}

impl ContextAwareConfig {
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.auto_detect_context = other.auto_detect_context;
        if other.force_context.is_some() {
            self.force_context = other.force_context;
        }
        self.enable_single_repo_optimizations = other.enable_single_repo_optimizations;
        self.enable_monorepo_features = other.enable_monorepo_features;
        if other.context_cache_duration.as_secs() > 0 {
            self.context_cache_duration = other.context_cache_duration;
        }
        Ok(())
    }
}

/// Configuration for performance optimizations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable parallel processing
    #[serde(default = "default_true")]
    pub enable_parallel_processing: bool,

    /// Maximum worker threads
    #[serde(default = "default_max_workers")]
    pub max_worker_threads: usize,

    /// Memory optimization level
    #[serde(default = "default_memory_optimization")]
    pub memory_optimization: MemoryOptimizationLevel,

    /// Enable I/O optimizations
    #[serde(default = "default_true")]
    pub enable_io_optimizations: bool,

    /// Batch processing size
    #[serde(default = "default_batch_size")]
    pub batch_processing_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_parallel_processing: true,
            max_worker_threads: default_max_workers(),
            memory_optimization: default_memory_optimization(),
            enable_io_optimizations: true,
            batch_processing_size: default_batch_size(),
        }
    }
}

impl PerformanceConfig {
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.enable_parallel_processing = other.enable_parallel_processing;
        if other.max_worker_threads > 0 {
            self.max_worker_threads = other.max_worker_threads;
        }
        self.memory_optimization = other.memory_optimization;
        self.enable_io_optimizations = other.enable_io_optimizations;
        if other.batch_processing_size > 0 {
            self.batch_processing_size = other.batch_processing_size;
        }
        Ok(())
    }
}

/// Configuration for caching behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default = "default_true")]
    pub enable_cache: bool,

    /// Cache time-to-live
    #[serde(default = "default_cache_ttl", with = "humantime_serde")]
    pub cache_ttl: Duration,

    /// Maximum cache size in MB
    #[serde(default = "default_max_cache_size")]
    pub max_cache_size_mb: usize,

    /// Cache cleanup interval
    #[serde(default = "default_cache_cleanup_interval", with = "humantime_serde")]
    pub cleanup_interval: Duration,

    /// Enable persistent cache
    #[serde(default = "default_true")]
    pub enable_persistent_cache: bool,

    /// Cache compression level
    #[serde(default = "default_cache_compression")]
    pub compression_level: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_ttl: default_cache_ttl(),
            max_cache_size_mb: default_max_cache_size(),
            cleanup_interval: default_cache_cleanup_interval(),
            enable_persistent_cache: true,
            compression_level: default_cache_compression(),
        }
    }
}

impl CacheConfig {
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.enable_cache = other.enable_cache;
        if other.cache_ttl.as_secs() > 0 {
            self.cache_ttl = other.cache_ttl;
        }
        if other.max_cache_size_mb > 0 {
            self.max_cache_size_mb = other.max_cache_size_mb;
        }
        if other.cleanup_interval.as_secs() > 0 {
            self.cleanup_interval = other.cleanup_interval;
        }
        self.enable_persistent_cache = other.enable_persistent_cache;
        self.compression_level = other.compression_level;
        Ok(())
    }
}

// Enums

/// Version bump strategies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VersionBumpStrategy {
    /// Bump major version (breaking changes)
    Major,
    /// Bump minor version (new features)
    Minor,
    /// Bump patch version (bug fixes)
    Patch,
    /// Create snapshot version with timestamp/SHA
    Snapshot,
    /// Cascade bump across dependencies
    Cascade,
}

/// Affected package detection strategies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AffectedDetectionStrategy {
    /// Automatically detect from changes
    Auto,
    /// Manual specification required
    Manual,
    /// Use git diff to detect changes
    GitDiff,
    /// Use dependency graph analysis
    DependencyGraph,
}

/// Supported dependency protocols.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyProtocol {
    /// Standard npm registry
    Npm,
    /// JSR registry
    Jsr,
    /// Git repositories
    Git,
    /// Local file paths
    File,
    /// Workspace protocols
    Workspace,
    /// Direct URLs
    Url,
    /// GitHub shortcuts
    GitHub,
}

/// Circular dependency handling strategies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircularDependencyHandling {
    /// Warn about circular dependencies
    Warn,
    /// Error on circular dependencies
    Error,
    /// Ignore circular dependencies
    Ignore,
}

/// Project context types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectContextType {
    /// Single repository
    SingleRepository,
    /// Monorepo/workspace
    Monorepo,
}

/// Memory optimization levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryOptimizationLevel {
    /// No optimization (highest memory usage)
    None,
    /// Basic optimization
    Basic,
    /// Aggressive optimization (lowest memory usage)
    Aggressive,
}

// Default value functions

fn default_config_version() -> String {
    "1.0".to_string()
}

fn default_version_bump_strategy() -> VersionBumpStrategy {
    // Check environment variable
    if let Ok(env_strategy) = std::env::var("SUBLIME_PKG_VERSION_BUMP_STRATEGY") {
        match env_strategy.trim().to_lowercase().as_str() {
            "major" => return VersionBumpStrategy::Major,
            "minor" => return VersionBumpStrategy::Minor,
            "patch" => return VersionBumpStrategy::Patch,
            "snapshot" => return VersionBumpStrategy::Snapshot,
            "cascade" => return VersionBumpStrategy::Cascade,
            _ => {} // Fall through to default
        }
    }
    VersionBumpStrategy::Patch
}

fn default_cascade_bumping() -> bool {
    // Check environment variable
    if let Ok(env_cascade) = std::env::var("SUBLIME_PKG_CASCADE_BUMPING") {
        return env_cascade.trim().to_lowercase() == "true";
    }
    false
}

fn default_snapshot_prefix() -> String {
    // Check environment variable
    if let Ok(env_prefix) = std::env::var("SUBLIME_PKG_SNAPSHOT_PREFIX") {
        let prefix = env_prefix.trim();
        if !prefix.is_empty() {
            return prefix.to_string();
        }
    }
    "snapshot".to_string()
}

fn default_affected_detection() -> AffectedDetectionStrategy {
    // Check environment variable
    if let Ok(env_detection) = std::env::var("SUBLIME_PKG_AFFECTED_DETECTION") {
        match env_detection.trim().to_lowercase().as_str() {
            "auto" => return AffectedDetectionStrategy::Auto,
            "manual" => return AffectedDetectionStrategy::Manual,
            "git-diff" => return AffectedDetectionStrategy::GitDiff,
            "dependency-graph" => return AffectedDetectionStrategy::DependencyGraph,
            _ => {} // Fall through to default
        }
    }
    AffectedDetectionStrategy::Auto
}

fn default_concurrent_downloads() -> usize {
    // Check environment variable
    if let Ok(env_concurrent) = std::env::var("SUBLIME_PKG_CONCURRENT_DOWNLOADS") {
        if let Ok(concurrent) = env_concurrent.trim().parse::<usize>() {
            if (1..=50).contains(&concurrent) {
                return concurrent;
            }
        }
    }
    10
}

fn default_registry_timeout() -> Duration {
    // Check environment variable
    if let Ok(env_timeout) = std::env::var("SUBLIME_PKG_REGISTRY_TIMEOUT") {
        if let Ok(seconds) = env_timeout.trim().parse::<u64>() {
            if (1..=300).contains(&seconds) {
                return Duration::from_secs(seconds);
            }
        }
    }
    Duration::from_secs(30)
}

fn default_supported_protocols() -> Vec<DependencyProtocol> {
    // Check environment variable
    if let Ok(env_protocols) = std::env::var("SUBLIME_PKG_SUPPORTED_PROTOCOLS") {
        let mut protocols = Vec::new();
        for protocol_name in env_protocols.split(',') {
            match protocol_name.trim().to_lowercase().as_str() {
                "npm" => protocols.push(DependencyProtocol::Npm),
                "jsr" => protocols.push(DependencyProtocol::Jsr),
                "git" => protocols.push(DependencyProtocol::Git),
                "file" => protocols.push(DependencyProtocol::File),
                "workspace" => protocols.push(DependencyProtocol::Workspace),
                "url" => protocols.push(DependencyProtocol::Url),
                "github" => protocols.push(DependencyProtocol::GitHub),
                _ => {} // Ignore unknown protocols
            }
        }
        if !protocols.is_empty() {
            return protocols;
        }
    }
    
    vec![
        DependencyProtocol::Npm,
        DependencyProtocol::Jsr,
        DependencyProtocol::Git,
        DependencyProtocol::File,
        DependencyProtocol::Workspace,
        DependencyProtocol::Url,
        DependencyProtocol::GitHub,
    ]
}

fn default_fallback_registries() -> Vec<String> {
    vec![
        "https://registry.npmjs.org".to_string(),
        "https://jsr.io".to_string(),
    ]
}

fn default_circular_handling() -> CircularDependencyHandling {
    // Check environment variable
    if let Ok(env_handling) = std::env::var("SUBLIME_PKG_CIRCULAR_DEP_HANDLING") {
        match env_handling.trim().to_lowercase().as_str() {
            "warn" => return CircularDependencyHandling::Warn,
            "error" => return CircularDependencyHandling::Error,
            "ignore" => return CircularDependencyHandling::Ignore,
            _ => {} // Fall through to default
        }
    }
    CircularDependencyHandling::Warn
}

fn default_max_cycle_depth() -> usize {
    // Check environment variable
    if let Ok(env_depth) = std::env::var("SUBLIME_PKG_MAX_CYCLE_DEPTH") {
        if let Ok(depth) = env_depth.trim().parse::<usize>() {
            if (1..=20).contains(&depth) {
                return depth;
            }
        }
    }
    10
}

fn default_context_cache_duration() -> Duration {
    Duration::from_secs(300) // 5 minutes
}

fn default_max_workers() -> usize {
    num_cpus::get().clamp(1, 16)
}

fn default_memory_optimization() -> MemoryOptimizationLevel {
    MemoryOptimizationLevel::Basic
}

fn default_batch_size() -> usize {
    100
}

fn default_cache_ttl() -> Duration {
    // Check environment variable
    if let Ok(env_ttl) = std::env::var("SUBLIME_PKG_CACHE_TTL") {
        if let Ok(seconds) = env_ttl.trim().parse::<u64>() {
            if (60..=86400).contains(&seconds) { // 1 minute to 1 day
                return Duration::from_secs(seconds);
            }
        }
    }
    Duration::from_secs(3600) // 1 hour
}

fn default_max_cache_size() -> usize {
    100 // 100 MB
}

fn default_cache_cleanup_interval() -> Duration {
    Duration::from_secs(1800) // 30 minutes
}

fn default_cache_compression() -> u32 {
    6 // Balanced compression level
}

fn default_true() -> bool {
    true
}