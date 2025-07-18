//! # Project Configuration Types
//!
//! ## What
//! This module defines configuration-related types for project detection
//! and validation, including config scopes, formats, and values.
//!
//! ## How
//! Configuration types provide a flexible system for managing project
//! settings across different scopes and formats.
//!
//! ## Why
//! Separate configuration types enable comprehensive project configuration
//! with clear hierarchy and format support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration options for project detection and validation.
///
/// This struct controls how projects are detected and validated,
/// allowing fine-grained control over the detection process.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectConfig;
///
/// let config = ProjectConfig::new()
///     .with_detect_package_manager(true)
///     .with_validate_structure(true)
///     .with_detect_monorepo(true);
/// ```
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Root directory for project detection (None uses current directory)
    pub(crate) root: Option<PathBuf>,
    /// Whether to detect the package manager
    pub(crate) detect_package_manager: bool,
    /// Whether to validate project structure
    pub(crate) validate_structure: bool,
    /// Whether to detect monorepo structures
    pub(crate) detect_monorepo: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectConfig {
    /// Creates a new `ProjectConfig` with default values.
    ///
    /// Default settings:
    /// - No specified root (uses current directory)
    /// - Package manager detection enabled
    /// - Structure validation enabled
    /// - Monorepo detection enabled
    ///
    /// # Returns
    ///
    /// A new `ProjectConfig` with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: None,
            detect_package_manager: true,
            validate_structure: true,
            detect_monorepo: true,
        }
    }

    /// Sets the root directory for project detection.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory to use
    ///
    /// # Returns
    ///
    /// Self with the root directory updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_root("/path/to/project");
    /// ```
    #[must_use]
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Sets whether to detect package managers.
    ///
    /// # Arguments
    ///
    /// * `detect` - Whether to detect package managers
    ///
    /// # Returns
    ///
    /// Self with the package manager detection setting updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_detect_package_manager(false);
    /// ```
    #[must_use]
    pub fn with_detect_package_manager(mut self, detect: bool) -> Self {
        self.detect_package_manager = detect;
        self
    }

    /// Sets whether to validate project structure.
    ///
    /// # Arguments
    ///
    /// * `validate` - Whether to validate project structure
    ///
    /// # Returns
    ///
    /// Self with the validation setting updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_validate_structure(false);
    /// ```
    #[must_use]
    pub fn with_validate_structure(mut self, validate: bool) -> Self {
        self.validate_structure = validate;
        self
    }

    /// Sets whether to detect monorepo structures.
    ///
    /// # Arguments
    ///
    /// * `detect` - Whether to detect monorepo structures
    ///
    /// # Returns
    ///
    /// Self with the monorepo detection setting updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_detect_monorepo(false);
    /// ```
    #[must_use]
    pub fn with_detect_monorepo(mut self, detect: bool) -> Self {
        self.detect_monorepo = detect;
        self
    }
}

/// Configuration scope levels.
///
/// This enum defines the different scopes that configuration can apply to,
/// from the most global (system-wide) to the most specific (runtime only).
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ConfigScope;
///
/// // Accessing different configuration scopes
/// let global = ConfigScope::Global;
/// let user = ConfigScope::User;
/// let project = ConfigScope::Project;
/// let runtime = ConfigScope::Runtime;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigScope {
    /// Global configuration (system-wide)
    Global,
    /// User configuration (user-specific)
    User,
    /// Project configuration (project-specific)
    Project,
    /// Runtime configuration (in-memory only)
    Runtime,
}

/// Configuration file formats.
///
/// This enum defines the supported file formats for configuration files,
/// allowing the system to properly parse and serialize configuration data.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ConfigFormat;
///
/// // Supporting different configuration formats
/// let json_format = ConfigFormat::Json;
/// let toml_format = ConfigFormat::Toml;
/// let yaml_format = ConfigFormat::Yaml;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
}

/// A configuration value that can represent different data types.
///
/// This enum provides a flexible way to store and manipulate configuration values
/// of various types, including strings, numbers, booleans, arrays, and nested maps.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use sublime_standard_tools::project::ConfigValue;
///
/// let string_val = ConfigValue::String("hello".to_string());
/// let int_val = ConfigValue::Integer(42);
/// let bool_val = ConfigValue::Boolean(true);
/// let array_val = ConfigValue::Array(vec![string_val, int_val]);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<ConfigValue>),
    /// Map of values
    Map(HashMap<String, ConfigValue>),
    /// Null value
    Null,
}

impl ConfigValue {
    /// Checks if this value is a string.
    ///
    /// # Returns
    ///
    /// `true` if this is a string value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert!(val.is_string());
    /// ```
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Checks if this value is an integer.
    ///
    /// # Returns
    ///
    /// `true` if this is an integer value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Integer(42);
    /// assert!(val.is_integer());
    /// ```
    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    /// Checks if this value is a float.
    ///
    /// # Returns
    ///
    /// `true` if this is a float value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Float(3.14);
    /// assert!(val.is_float());
    /// ```
    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    /// Checks if this value is a boolean.
    ///
    /// # Returns
    ///
    /// `true` if this is a boolean value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Boolean(true);
    /// assert!(val.is_boolean());
    /// ```
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    /// Checks if this value is an array.
    ///
    /// # Returns
    ///
    /// `true` if this is an array value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Array(vec![]);
    /// assert!(val.is_array());
    /// ```
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Checks if this value is a map.
    ///
    /// # Returns
    ///
    /// `true` if this is a map value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Map(HashMap::new());
    /// assert!(val.is_map());
    /// ```
    #[must_use]
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    /// Checks if this value is null.
    ///
    /// # Returns
    ///
    /// `true` if this is a null value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Null;
    /// assert!(val.is_null());
    /// ```
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Extracts the string value if this is a string.
    ///
    /// # Returns
    ///
    /// * `Some(&str)` - If this is a string value
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert_eq!(val.as_string(), Some("hello"));
    /// ```
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the integer value if this is an integer.
    ///
    /// # Returns
    ///
    /// * `Some(i64)` - If this is an integer value
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Integer(42);
    /// assert_eq!(val.as_integer(), Some(42));
    /// ```
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Extracts the float value if this is a float.
    ///
    /// This method also returns integer values converted to floats.
    ///
    /// # Returns
    ///
    /// * `Some(f64)` - If this is a float or integer value
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Float(3.14);
    /// assert_eq!(val.as_float(), Some(3.14));
    /// 
    /// // Integer values are converted to floats
    /// let val = ConfigValue::Integer(42);
    /// assert_eq!(val.as_float(), Some(42.0));
    /// ```
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Extracts the boolean value if this is a boolean.
    ///
    /// # Returns
    ///
    /// * `Some(bool)` - If this is a boolean value
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Boolean(true);
    /// assert_eq!(val.as_boolean(), Some(true));
    /// ```
    #[must_use]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Extracts the array value if this is an array.
    ///
    /// # Returns
    ///
    /// * `Some(&[ConfigValue])` - If this is an array value
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Array(vec![ConfigValue::String("hello".to_string())]);
    /// assert!(val.as_array().is_some());
    /// ```
    #[must_use]
    pub fn as_array(&self) -> Option<&[ConfigValue]> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Extracts the map value if this is a map.
    ///
    /// # Returns
    ///
    /// * `Some(&HashMap<String, ConfigValue>)` - If this is a map value
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sublime_standard_tools::project::ConfigValue;
    ///
    /// let val = ConfigValue::Map(HashMap::new());
    /// assert!(val.as_map().is_some());
    /// ```
    #[must_use]
    pub fn as_map(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            Self::Map(map) => Some(map),
            _ => None,
        }
    }
}