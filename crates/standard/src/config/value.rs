//! Configuration value types.
//!
//! This module provides a generic value type that can represent any configuration
//! value, similar to JSON but with additional type safety and convenience methods.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{ConfigError, ConfigResult};

/// Represents a configuration value.
///
/// This enum can hold any type of configuration value, providing a generic
/// way to represent configuration data from various sources.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::config::ConfigValue;
/// use std::collections::HashMap;
///
/// // Create different types of values
/// let string_val = ConfigValue::String("hello".to_string());
/// let int_val = ConfigValue::Integer(42);
/// let bool_val = ConfigValue::Boolean(true);
///
/// // Create nested structures
/// let mut map = HashMap::new();
/// map.insert("name".to_string(), ConfigValue::String("test".to_string()));
/// map.insert("count".to_string(), ConfigValue::Integer(10));
/// let map_val = ConfigValue::Map(map);
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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
    /// use sublime_standard_tools::config::ConfigValue;
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

    /// Gets a value from a map by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Some(&ConfigValue)` - If this is a map and the key exists
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), ConfigValue::String("value".to_string()));
    /// let val = ConfigValue::Map(map);
    /// assert_eq!(val.get("key").and_then(|v| v.as_string()), Some("value"));
    /// ```
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        match self {
            Self::Map(map) => map.get(key),
            _ => None,
        }
    }

    /// Gets a mutable value from a map by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Some(&mut ConfigValue)` - If this is a map and the key exists
    /// * `None` - Otherwise
    pub fn get_mut(&mut self, key: &str) -> Option<&mut ConfigValue> {
        match self {
            Self::Map(map) => map.get_mut(key),
            _ => None,
        }
    }

    /// Inserts a value into a map.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert at
    /// * `value` - The value to insert
    ///
    /// # Returns
    ///
    /// * `Ok(Option<ConfigValue>)` - The previous value if any
    /// * `Err(ConfigError)` - If this is not a map
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let mut val = ConfigValue::Map(HashMap::new());
    /// val.insert("key", ConfigValue::String("value".to_string())).unwrap();
    /// assert_eq!(val.get("key").and_then(|v| v.as_string()), Some("value"));
    /// ```
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: ConfigValue,
    ) -> ConfigResult<Option<ConfigValue>> {
        match self {
            Self::Map(map) => Ok(map.insert(key.into(), value)),
            _ => Err(ConfigError::type_error("map", self.type_name())),
        }
    }

    /// Gets the type name of this value.
    ///
    /// # Returns
    ///
    /// A string describing the type of this value.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// assert_eq!(ConfigValue::String("hello".to_string()).type_name(), "string");
    /// assert_eq!(ConfigValue::Integer(42).type_name(), "integer");
    /// assert_eq!(ConfigValue::Float(3.14).type_name(), "float");
    /// assert_eq!(ConfigValue::Boolean(true).type_name(), "boolean");
    /// assert_eq!(ConfigValue::Array(vec![]).type_name(), "array");
    /// assert_eq!(ConfigValue::Map(Default::default()).type_name(), "map");
    /// assert_eq!(ConfigValue::Null.type_name(), "null");
    /// ```
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::Boolean(_) => "boolean",
            Self::Array(_) => "array",
            Self::Map(_) => "map",
            Self::Null => "null",
        }
    }

    /// Merges another value into this one.
    ///
    /// For maps, this performs a deep merge. For other types, the other value
    /// simply replaces this one.
    ///
    /// # Arguments
    ///
    /// * `other` - The value to merge into this one
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let mut base = ConfigValue::Map(HashMap::new());
    /// base.insert("a", ConfigValue::Integer(1)).unwrap();
    /// base.insert("b", ConfigValue::Integer(2)).unwrap();
    ///
    /// let mut other = ConfigValue::Map(HashMap::new());
    /// other.insert("b", ConfigValue::Integer(3)).unwrap();
    /// other.insert("c", ConfigValue::Integer(4)).unwrap();
    ///
    /// base.merge(other);
    /// assert_eq!(base.get("a").and_then(|v| v.as_integer()), Some(1));
    /// assert_eq!(base.get("b").and_then(|v| v.as_integer()), Some(3));
    /// assert_eq!(base.get("c").and_then(|v| v.as_integer()), Some(4));
    /// ```
    pub fn merge(&mut self, other: ConfigValue) {
        match (self, other) {
            (Self::Map(base), Self::Map(other)) => {
                for (key, value) in other {
                    match base.get_mut(&key) {
                        Some(base_value) => base_value.merge(value),
                        None => {
                            base.insert(key, value);
                        }
                    }
                }
            }
            (base, other) => *base = other,
        }
    }
}

impl Default for ConfigValue {
    fn default() -> Self {
        Self::Null
    }
}

impl From<String> for ConfigValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for ConfigValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for ConfigValue {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<i32> for ConfigValue {
    fn from(i: i32) -> Self {
        Self::Integer(i64::from(i))
    }
}

impl From<f64> for ConfigValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<f32> for ConfigValue {
    fn from(f: f32) -> Self {
        Self::Float(f64::from(f))
    }
}

impl From<bool> for ConfigValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl<T: Into<ConfigValue>> From<Vec<T>> for ConfigValue {
    fn from(vec: Vec<T>) -> Self {
        Self::Array(vec.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<ConfigValue>> From<HashMap<String, T>> for ConfigValue {
    fn from(map: HashMap<String, T>) -> Self {
        Self::Map(map.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}
