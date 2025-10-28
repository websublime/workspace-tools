//! .npmrc file parser for NPM registry configuration.
//!
//! **What**: Provides parsing and resolution of .npmrc configuration files to extract
//! registry URLs, scoped registry mappings, and authentication tokens.
//!
//! **How**: This module reads .npmrc files from the workspace root and user home directory,
//! parses the key-value format with support for comments and environment variable substitution,
//! and merges configuration with workspace settings taking precedence.
//!
//! **Why**: To respect existing NPM registry configuration when detecting and applying
//! dependency upgrades, including private registries and authentication.
//!
//! # Features
//!
//! - **Workspace .npmrc**: Reads .npmrc from workspace root
//! - **User .npmrc**: Falls back to user home directory .npmrc
//! - **Configuration Merging**: Workspace config overrides user config
//! - **Scoped Registries**: Parses @scope:registry mappings
//! - **Authentication**: Extracts auth tokens with environment variable support
//! - **Comment Handling**: Ignores # and // style comments
//! - **Environment Variables**: Substitutes ${VAR_NAME} placeholders
//!
//! # .npmrc Format
//!
//! The .npmrc file uses a simple key=value format:
//!
//! ```text
//! # Default registry
//! registry=https://registry.npmjs.org
//!
//! # Scoped registry
//! @myorg:registry=https://npm.myorg.com
//!
//! # Authentication token with environment variable
//! //npm.myorg.com/:_authToken=${NPM_TOKEN}
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! // Load .npmrc configuration
//! let npmrc = NpmrcConfig::from_workspace(&workspace_root, &fs).await?;
//!
//! // Resolve registry for a package
//! if let Some(registry) = npmrc.resolve_registry("@myorg/package") {
//!     println!("Registry for @myorg/package: {}", registry);
//! }
//!
//! // Get authentication token for a registry
//! if let Some(token) = npmrc.get_auth_token("https://npm.myorg.com") {
//!     println!("Auth token found for private registry");
//! }
//! ```

use crate::error::UpgradeError;
use std::collections::HashMap;
use std::path::Path;
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Parsed .npmrc configuration.
///
/// Contains registry URLs, scoped registry mappings, authentication tokens,
/// and other configuration properties extracted from .npmrc files.
///
/// # Example
///
/// ```rust,ignore
/// let config = NpmrcConfig::default();
/// assert!(config.registry.is_none());
/// assert!(config.scoped_registries.is_empty());
/// ```
#[derive(Debug, Clone, Default)]
pub struct NpmrcConfig {
    /// Default registry URL.
    ///
    /// Extracted from `registry=<url>` line in .npmrc.
    pub registry: Option<String>,

    /// Scoped registry mappings.
    ///
    /// Maps scope names (without @) to registry URLs.
    /// Extracted from `@scope:registry=<url>` lines.
    pub scoped_registries: HashMap<String, String>,

    /// Authentication tokens for registries.
    ///
    /// Maps registry URLs (or URL patterns) to auth tokens.
    /// Extracted from `//<registry>/:_authToken=<token>` lines.
    pub auth_tokens: HashMap<String, String>,

    /// Other configuration properties.
    ///
    /// Stores any additional key-value pairs not specifically handled.
    pub other: HashMap<String, String>,
}

impl NpmrcConfig {
    /// Parses .npmrc files from workspace root and user home directory.
    ///
    /// Looks for .npmrc in the workspace root first, then in the user's home
    /// directory. Merges configuration with workspace .npmrc taking precedence
    /// over user .npmrc.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace
    /// * `fs` - Filesystem abstraction for reading files
    ///
    /// # Returns
    ///
    /// Parsed and merged .npmrc configuration.
    ///
    /// # Errors
    ///
    /// Returns `UpgradeError::NpmrcParseError` if:
    /// - .npmrc file exists but cannot be read
    /// - .npmrc contains invalid syntax
    ///
    /// Returns `Ok` with empty/default config if no .npmrc files are found.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let npmrc = NpmrcConfig::from_workspace(&workspace_root, &fs).await?;
    /// ```
    pub async fn from_workspace<F>(workspace_root: &Path, fs: &F) -> Result<Self, UpgradeError>
    where
        F: AsyncFileSystem,
    {
        let mut config = Self::default();

        // Try to load user .npmrc first (lower precedence)
        if let Some(home_dir) = dirs::home_dir() {
            let user_npmrc_path = home_dir.join(".npmrc");
            if fs.exists(&user_npmrc_path).await {
                match Self::parse_npmrc_file(&user_npmrc_path, fs).await {
                    Ok(user_config) => {
                        config.merge_with(user_config);
                    }
                    Err(e) => {
                        // Log but don't fail on user .npmrc parse errors
                        eprintln!("Warning: Failed to parse user .npmrc: {}", e);
                    }
                }
            }
        }

        // Load workspace .npmrc (higher precedence)
        let workspace_npmrc_path = workspace_root.join(".npmrc");
        if fs.exists(&workspace_npmrc_path).await {
            let workspace_config = Self::parse_npmrc_file(&workspace_npmrc_path, fs).await?;
            config.merge_with(workspace_config);
        }

        Ok(config)
    }

    /// Resolves registry URL for a package name.
    ///
    /// Returns scoped registry if package is scoped and a matching scope
    /// registry is configured. Otherwise returns the default registry.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package (may include scope)
    ///
    /// # Returns
    ///
    /// Registry URL if one is configured, None if no registry is configured.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::NpmrcConfig;
    /// use std::collections::HashMap;
    ///
    /// let mut config = NpmrcConfig::default();
    /// config.registry = Some("https://registry.npmjs.org".to_string());
    /// config.scoped_registries.insert(
    ///     "myorg".to_string(),
    ///     "https://npm.myorg.com".to_string()
    /// );
    ///
    /// // Scoped package uses scoped registry
    /// assert_eq!(
    ///     config.resolve_registry("@myorg/package"),
    ///     Some("https://npm.myorg.com")
    /// );
    ///
    /// // Unscoped package uses default registry
    /// assert_eq!(
    ///     config.resolve_registry("lodash"),
    ///     Some("https://registry.npmjs.org")
    /// );
    /// ```
    pub fn resolve_registry(&self, package_name: &str) -> Option<&str> {
        // Check if package is scoped (starts with @)
        if let Some(scope) = package_name.strip_prefix('@') {
            // Extract scope name (everything before the first '/')
            if let Some(scope_end) = scope.find('/') {
                let scope_name = &scope[..scope_end];

                // Check if we have a registry configured for this scope
                if let Some(registry) = self.scoped_registries.get(scope_name) {
                    return Some(registry.as_str());
                }
            }
        }

        // Fall back to default registry
        self.registry.as_deref()
    }

    /// Gets authentication token for a registry URL.
    ///
    /// Checks the auth_tokens map for a matching token. Tries exact match first,
    /// then normalized URL without protocol/trailing slashes.
    ///
    /// # Arguments
    ///
    /// * `registry_url` - The registry URL to check
    ///
    /// # Returns
    ///
    /// Authentication token if one is configured, None otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::NpmrcConfig;
    /// use std::collections::HashMap;
    ///
    /// let mut config = NpmrcConfig::default();
    /// config.auth_tokens.insert(
    ///     "npm.myorg.com".to_string(),
    ///     "npm_AbCdEf123456".to_string()
    /// );
    ///
    /// // Exact match
    /// assert_eq!(
    ///     config.get_auth_token("npm.myorg.com"),
    ///     Some("npm_AbCdEf123456")
    /// );
    ///
    /// // Match with protocol
    /// assert_eq!(
    ///     config.get_auth_token("https://npm.myorg.com"),
    ///     Some("npm_AbCdEf123456")
    /// );
    /// ```
    pub fn get_auth_token(&self, registry_url: &str) -> Option<&str> {
        // Try exact match first
        if let Some(token) = self.auth_tokens.get(registry_url) {
            return Some(token.as_str());
        }

        // Normalize URL for matching: remove protocol and trailing slashes
        let normalized = registry_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');

        // Try normalized match
        if let Some(token) = self.auth_tokens.get(normalized) {
            return Some(token.as_str());
        }

        // Try checking if any stored key matches the normalized URL
        for (key, token) in &self.auth_tokens {
            let normalized_key = key
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_start_matches("//")
                .trim_end_matches('/');

            if normalized_key == normalized {
                return Some(token.as_str());
            }
        }

        None
    }

    /// Parses a single .npmrc file.
    ///
    /// Reads the file content and parses line by line, handling comments,
    /// environment variable substitution, and different key formats.
    async fn parse_npmrc_file<F>(path: &Path, fs: &F) -> Result<Self, UpgradeError>
    where
        F: AsyncFileSystem,
    {
        let content =
            fs.read_file_string(path).await.map_err(|e| UpgradeError::NpmrcParseError {
                path: path.to_path_buf(),
                reason: format!("Failed to read file: {}", e),
            })?;

        Self::parse_content(&content, path)
    }

    /// Parses .npmrc content from a string.
    ///
    /// Processes each line, extracting registry configuration, scoped registries,
    /// authentication tokens, and other properties.
    fn parse_content(content: &str, path: &Path) -> Result<Self, UpgradeError> {
        let mut config = Self::default();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and full-line comments (# only, not //)
            // Note: // at the start is used for auth tokens like //registry.com/:_authToken
            if line.is_empty() || (line.starts_with('#') && !line.contains('=')) {
                continue;
            }

            // Parse key=value
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value_with_comment = &line[eq_pos + 1..];

                // Remove inline comments from the value only
                let value = Self::remove_comment(value_with_comment).trim();

                // Skip empty values
                if value.is_empty() {
                    continue;
                }

                // Substitute environment variables
                let value = Self::substitute_env_vars(value);

                // Parse different key formats
                if let Err(e) = Self::parse_key_value(&mut config, key, &value) {
                    return Err(UpgradeError::NpmrcParseError {
                        path: path.to_path_buf(),
                        reason: format!("Line {}: {}", line_num + 1, e),
                    });
                }
            }
        }

        Ok(config)
    }

    /// Removes comments from a value string.
    ///
    /// Supports both # and // style comments, but only when they appear
    /// after whitespace (not as part of URLs like https://).
    fn remove_comment(value: &str) -> &str {
        // Find # comment
        let hash_pos = value.find('#');

        // Find // comment that's preceded by whitespace
        let double_slash_pos = Self::find_comment_double_slash(value);

        match (hash_pos, double_slash_pos) {
            (Some(h), Some(d)) => &value[..h.min(d)],
            (Some(h), None) => &value[..h],
            (None, Some(d)) => &value[..d],
            (None, None) => value,
        }
    }

    /// Finds // that is a comment, not part of a URL.
    ///
    /// Only treats // as a comment if it's preceded by whitespace.
    /// This avoids treating http:// and https:// as comments.
    fn find_comment_double_slash(value: &str) -> Option<usize> {
        let mut pos = 0;
        while let Some(idx) = value[pos..].find("//") {
            let absolute_pos = pos + idx;

            // Check if preceded by whitespace
            if absolute_pos > 0
                && let Some(prev_char) = value[..absolute_pos].chars().last()
                    && prev_char.is_whitespace() {
                        return Some(absolute_pos);
                    }

            // Move past this // and continue searching
            pos = absolute_pos + 2;
        }

        None
    }

    /// Substitutes environment variables in a value.
    ///
    /// Replaces ${VAR_NAME} with the value of the environment variable.
    /// If the variable is not set, keeps the placeholder.
    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();

        // Find all ${...} patterns
        while let Some(start) = result.find("${") {
            if let Some(end) = result[start..].find('}') {
                let var_name = &result[start + 2..start + end];

                // Get environment variable value
                if let Ok(var_value) = std::env::var(var_name) {
                    // Replace ${VAR_NAME} with value
                    result.replace_range(start..start + end + 1, &var_value);
                } else {
                    // Variable not set, move past this placeholder
                    // to avoid infinite loop
                    break;
                }
            } else {
                // No closing }, stop processing
                break;
            }
        }

        result
    }

    /// Parses a single key-value pair and updates the config.
    fn parse_key_value(config: &mut Self, key: &str, value: &str) -> Result<(), String> {
        // Check for scoped registry: @scope:registry
        if let Some(scope_key) = key.strip_prefix('@')
            && let Some(colon_pos) = scope_key.find(':') {
                let scope_name = &scope_key[..colon_pos];
                let property = &scope_key[colon_pos + 1..];

                if property == "registry" {
                    config.scoped_registries.insert(scope_name.to_string(), value.to_string());
                    return Ok(());
                }
            }

        // Check for auth token: //<registry>/:_authToken
        if key.starts_with("//") && key.contains(":_authToken") {
            // Extract registry URL from key (between // and /:_authToken)
            if let Some(auth_pos) = key.find("/:_authToken") {
                let registry = &key[2..auth_pos];
                config.auth_tokens.insert(registry.to_string(), value.to_string());
                return Ok(());
            } else if let Some(auth_pos) = key.find(":_authToken") {
                // Handle case without leading slash: //registry.com:_authToken
                let registry = &key[2..auth_pos];
                config.auth_tokens.insert(registry.to_string(), value.to_string());
                return Ok(());
            }
        }

        // Check for other auth formats
        if key.ends_with(":_authToken") {
            let registry = key.trim_end_matches(":_authToken").trim_end_matches('/');
            config.auth_tokens.insert(registry.to_string(), value.to_string());
            return Ok(());
        }

        // Check for default registry
        if key == "registry" {
            config.registry = Some(value.to_string());
            return Ok(());
        }

        // Store other properties
        config.other.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Merges another config into this one.
    ///
    /// Properties from `other` override properties in `self`.
    pub(crate) fn merge_with(&mut self, other: Self) {
        // Other registry overrides self registry
        if other.registry.is_some() {
            self.registry = other.registry;
        }

        // Merge scoped registries (other takes precedence)
        self.scoped_registries.extend(other.scoped_registries);

        // Merge auth tokens (other takes precedence)
        self.auth_tokens.extend(other.auth_tokens);

        // Merge other properties (other takes precedence)
        self.other.extend(other.other);
    }
}
