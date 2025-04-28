//
// Standard Validation Rules
//

use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use super::{ValidationResult, ValidationRule};

/// String length validation rule
///
/// Checks if a string's length is within the specified bounds.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::validation::{StringLengthRule, ValidationRule};
///
/// let rule = StringLengthRule::new(3, 10);
/// assert!(rule.validate(&"hello".to_string()).is_valid());
/// assert!(!rule.validate(&"hi".to_string()).is_valid());
/// assert!(!rule.validate(&"this is too long".to_string()).is_valid());
/// ```
#[derive(Debug, Clone)]
pub struct StringLengthRule {
    /// Minimum allowed length (inclusive)
    min_length: usize,
    /// Maximum allowed length (inclusive)
    max_length: usize,
    /// Whether to treat empty strings as valid
    allow_empty: bool,
}

impl StringLengthRule {
    /// Creates a new string length validation rule
    ///
    /// # Arguments
    ///
    /// * `min_length` - Minimum allowed length (inclusive)
    /// * `max_length` - Maximum allowed length (inclusive)
    ///
    /// # Returns
    ///
    /// A new string length validation rule
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::StringLengthRule;
    ///
    /// let rule = StringLengthRule::new(5, 50);
    /// ```
    #[must_use]
    pub fn new(min_length: usize, max_length: usize) -> Self {
        Self { min_length, max_length, allow_empty: false }
    }

    /// Allows empty strings to be considered valid
    ///
    /// # Returns
    ///
    /// A rule that considers empty strings valid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::{StringLengthRule, ValidationRule};
    ///
    /// let rule = StringLengthRule::new(5, 50).allow_empty();
    /// assert!(rule.validate(&"".to_string()).is_valid());
    /// ```
    #[must_use]
    pub fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }
}

impl ValidationRule<String> for StringLengthRule {
    fn validate(&self, target: &String) -> ValidationResult {
        // Special case for empty strings
        if target.is_empty() {
            return if self.allow_empty {
                ValidationResult::Valid
            } else {
                ValidationResult::Error(vec!["String cannot be empty".to_string()])
            };
        }

        let length = target.len();
        if length < self.min_length {
            ValidationResult::Error(vec![format!(
                "String too short (minimum length is {})",
                self.min_length
            )])
        } else if length > self.max_length {
            ValidationResult::Error(vec![format!(
                "String too long (maximum length is {})",
                self.max_length
            )])
        } else {
            ValidationResult::Valid
        }
    }
}

impl ValidationRule<&str> for StringLengthRule {
    fn validate(&self, target: &&str) -> ValidationResult {
        self.validate(&(*target).to_string())
    }
}

/// Pattern validation rule
///
/// Validates that a string matches a specified pattern.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::validation::{PatternRule, ValidationRule};
///
/// // Email validation rule
/// let email_rule = PatternRule::email();
/// assert!(email_rule.validate(&"user@example.com".to_string()).is_valid());
/// assert!(!email_rule.validate(&"not-an-email".to_string()).is_valid());
/// ```
#[derive(Debug, Clone)]
pub struct PatternRule {
    /// Pattern description for error messages
    pattern_name: String,
    /// Regular expression pattern to match
    pattern: String,
    /// Whether to negate the pattern match
    negate: bool,
}

impl PatternRule {
    /// Creates a new pattern validation rule
    ///
    /// # Arguments
    ///
    /// * `pattern_name` - Description of the pattern for error messages
    /// * `pattern` - Regular expression pattern to match
    ///
    /// # Returns
    ///
    /// A new pattern validation rule
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::PatternRule;
    ///
    /// // Create a rule for alphanumeric values
    /// let rule = PatternRule::new("alphanumeric", r"^[a-zA-Z0-9]+$");
    /// ```
    #[must_use]
    pub fn new(pattern_name: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self { pattern_name: pattern_name.into(), pattern: pattern.into(), negate: false }
    }

    /// Creates a rule validating email addresses
    ///
    /// # Returns
    ///
    /// A pattern rule for validating email addresses
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::{PatternRule, ValidationRule};
    ///
    /// let rule = PatternRule::email();
    /// assert!(rule.validate(&"user@example.com".to_string()).is_valid());
    /// assert!(!rule.validate(&"invalid-email".to_string()).is_valid());
    /// ```
    #[must_use]
    pub fn email() -> Self {
        Self::new("email", r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$")
    }

    /// Creates a rule validating URLs
    ///
    /// # Returns
    ///
    /// A pattern rule for validating URLs
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::{PatternRule, ValidationRule};
    ///
    /// let rule = PatternRule::url();
    /// assert!(rule.validate(&"https://example.com".to_string()).is_valid());
    /// assert!(!rule.validate(&"not a url".to_string()).is_valid());
    /// ```
    #[must_use]
    pub fn url() -> Self {
        Self::new("URL", r"^(https?|ftp)://[^\s/$.?#].[^\s]*$")
    }

    /// Creates a rule validating alphanumeric strings
    ///
    /// # Returns
    ///
    /// A pattern rule for validating alphanumeric strings
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::{PatternRule, ValidationRule};
    ///
    /// let rule = PatternRule::alphanumeric();
    /// assert!(rule.validate(&"abc123".to_string()).is_valid());
    /// assert!(!rule.validate(&"abc-123".to_string()).is_valid());
    /// ```
    #[must_use]
    pub fn alphanumeric() -> Self {
        Self::new("alphanumeric", r"^[a-zA-Z0-9]+$")
    }

    /// Creates a rule validating semver version strings
    ///
    /// # Returns
    ///
    /// A pattern rule for validating semver version strings
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::{PatternRule, ValidationRule};
    ///
    /// let rule = PatternRule::semver();
    /// assert!(rule.validate(&"1.2.3".to_string()).is_valid());
    /// assert!(rule.validate(&"1.2.3-beta.1".to_string()).is_valid());
    /// assert!(!rule.validate(&"1.2".to_string()).is_valid());
    /// ```
    #[must_use]
    pub fn semver() -> Self {
        Self::new(
            "semantic version",
            r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$",
        )
    }

    /// Inverts the validation logic (fails when pattern matches)
    ///
    /// # Returns
    ///
    /// A rule that fails when the pattern matches
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::{PatternRule, ValidationRule};
    ///
    /// // Create a rule that fails when a string contains digits
    /// let rule = PatternRule::new("digits", r"\d+").negate();
    /// assert!(rule.validate(&"abcdef".to_string()).is_valid());
    /// assert!(!rule.validate(&"abc123".to_string()).is_valid());
    /// ```
    #[must_use]
    pub fn negate(mut self) -> Self {
        self.negate = true;
        self
    }

    fn check_pattern(&self, target: &str) -> bool {
        let regex_result =
            regex::Regex::new(&self.pattern).map(|re| re.is_match(target)).unwrap_or(false);

        if self.negate {
            !regex_result
        } else {
            regex_result
        }
    }
}

impl ValidationRule<String> for PatternRule {
    fn validate(&self, target: &String) -> ValidationResult {
        if self.check_pattern(target) {
            ValidationResult::Valid
        } else {
            let msg = if self.negate {
                format!("String should not match {} pattern", self.pattern_name)
            } else {
                format!("String does not match {} pattern", self.pattern_name)
            };
            ValidationResult::Error(vec![msg])
        }
    }
}

impl ValidationRule<&str> for PatternRule {
    fn validate(&self, target: &&str) -> ValidationResult {
        if self.check_pattern(target) {
            ValidationResult::Valid
        } else {
            let msg = if self.negate {
                format!("String should not match {} pattern", self.pattern_name)
            } else {
                format!("String does not match {} pattern", self.pattern_name)
            };
            ValidationResult::Error(vec![msg])
        }
    }
}

/// Numeric range validation rule
///
/// Validates that a number is within a specified range.
///
/// # Type Parameters
///
/// * `T` - The numeric type to validate (e.g., i32, f64)
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::validation::{NumericRangeRule, ValidationRule};
///
/// let rule = NumericRangeRule::new(0, 100);
/// assert!(rule.validate(&50).is_valid());
/// assert!(!rule.validate(&-10).is_valid());
/// assert!(!rule.validate(&200).is_valid());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NumericRangeRule<T> {
    /// Minimum allowed value (inclusive)
    min: T,
    /// Maximum allowed value (inclusive)
    max: T,
}

impl<T: PartialOrd + Display + Copy> NumericRangeRule<T> {
    /// Creates a new numeric range validation rule
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum allowed value (inclusive)
    /// * `max` - Maximum allowed value (inclusive)
    ///
    /// # Returns
    ///
    /// A new numeric range validation rule
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::NumericRangeRule;
    ///
    /// // Integer range
    /// let int_rule = NumericRangeRule::new(0, 100);
    ///
    /// // Float range
    /// let float_rule = NumericRangeRule::new(0.0, 1.0);
    /// ```
    #[must_use]
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T: PartialOrd + Display + Copy> ValidationRule<T> for NumericRangeRule<T> {
    fn validate(&self, target: &T) -> ValidationResult {
        if *target < self.min {
            ValidationResult::Error(vec![format!("Value {} is below minimum {}", target, self.min)])
        } else if *target > self.max {
            ValidationResult::Error(vec![format!("Value {} is above maximum {}", target, self.max)])
        } else {
            ValidationResult::Valid
        }
    }
}

/// Parseable string validation rule
///
/// Validates that a string can be parsed into a specific type.
///
/// # Type Parameters
///
/// * `T` - The type to parse the string into
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::validation::{ParseableRule, ValidationRule};
///
/// let rule = ParseableRule::<i32>::new("integer");
/// assert!(rule.validate(&"42".to_string()).is_valid());
/// assert!(!rule.validate(&"not a number".to_string()).is_valid());
/// ```
#[derive(Debug, Clone)]
pub struct ParseableRule<T> {
    /// Name of the type for error messages
    type_name: String,
    /// Phantom marker for the target type
    _marker: std::marker::PhantomData<T>,
}

impl<T: FromStr> ParseableRule<T> {
    /// Creates a new parseable string validation rule
    ///
    /// # Arguments
    ///
    /// * `type_name` - Name of the type for error messages
    ///
    /// # Returns
    ///
    /// A new parseable string validation rule
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::ParseableRule;
    ///
    /// // Integer parsing rule
    /// let int_rule = ParseableRule::<i32>::new("integer");
    ///
    /// // Float parsing rule
    /// let float_rule = ParseableRule::<f64>::new("floating-point number");
    /// ```
    #[must_use]
    pub fn new(type_name: impl Into<String>) -> Self {
        Self { type_name: type_name.into(), _marker: std::marker::PhantomData }
    }
}

impl<T: FromStr> ValidationRule<String> for ParseableRule<T> {
    fn validate(&self, target: &String) -> ValidationResult {
        match target.parse::<T>() {
            Ok(_) => ValidationResult::Valid,
            Err(_) => ValidationResult::Error(vec![format!(
                "Value cannot be parsed as a {}",
                self.type_name
            )]),
        }
    }
}

impl<T: FromStr> ValidationRule<&str> for ParseableRule<T> {
    fn validate(&self, target: &&str) -> ValidationResult {
        match target.parse::<T>() {
            Ok(_) => ValidationResult::Valid,
            Err(_) => ValidationResult::Error(vec![format!(
                "Value cannot be parsed as a {}",
                self.type_name
            )]),
        }
    }
}

/// Path existence validation rule
///
/// Validates that a path exists and is of the expected type.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::validation::{PathExistsRule, ValidationRule};
/// use std::path::PathBuf;
///
/// let rule = PathExistsRule::exists();
/// let exists = std::env::current_dir().unwrap();
/// assert!(rule.validate(&exists).is_valid());
///
/// let does_not_exist = PathBuf::from("/path/that/does/not/exist");
/// assert!(!rule.validate(&does_not_exist).is_valid());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PathExistsRule {
    /// Whether to check if the path is a file
    require_file: bool,
    /// Whether to check if the path is a directory
    require_directory: bool,
}

impl PathExistsRule {
    /// Creates a rule that validates a path exists
    ///
    /// # Returns
    ///
    /// A path existence validation rule
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::PathExistsRule;
    ///
    /// let rule = PathExistsRule::exists();
    /// ```
    #[must_use]
    pub fn exists() -> Self {
        Self { require_file: false, require_directory: false }
    }

    /// Creates a rule that validates a path exists and is a file
    ///
    /// # Returns
    ///
    /// A file existence validation rule
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::PathExistsRule;
    ///
    /// let rule = PathExistsRule::file();
    /// ```
    #[must_use]
    pub fn file() -> Self {
        Self { require_file: true, require_directory: false }
    }

    /// Creates a rule that validates a path exists and is a directory
    ///
    /// # Returns
    ///
    /// A directory existence validation rule
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::PathExistsRule;
    ///
    /// let rule = PathExistsRule::directory();
    /// ```
    #[must_use]
    pub fn directory() -> Self {
        Self { require_file: false, require_directory: true }
    }
}

impl ValidationRule<PathBuf> for PathExistsRule {
    fn validate(&self, target: &PathBuf) -> ValidationResult {
        if !target.exists() {
            return ValidationResult::Error(vec![format!(
                "Path does not exist: {}",
                target.display()
            )]);
        }

        if self.require_file && !target.is_file() {
            return ValidationResult::Error(vec![format!(
                "Path is not a file: {}",
                target.display()
            )]);
        }

        if self.require_directory && !target.is_dir() {
            return ValidationResult::Error(vec![format!(
                "Path is not a directory: {}",
                target.display()
            )]);
        }

        ValidationResult::Valid
    }
}

impl ValidationRule<&Path> for PathExistsRule {
    fn validate(&self, target: &&Path) -> ValidationResult {
        if !target.exists() {
            return ValidationResult::Error(vec![format!(
                "Path does not exist: {}",
                target.display()
            )]);
        }

        if self.require_file && !target.is_file() {
            return ValidationResult::Error(vec![format!(
                "Path is not a file: {}",
                target.display()
            )]);
        }

        if self.require_directory && !target.is_dir() {
            return ValidationResult::Error(vec![format!(
                "Path is not a directory: {}",
                target.display()
            )]);
        }

        ValidationResult::Valid
    }
}

/// Combined validation rule
///
/// Combines multiple validation rules into a single rule.
///
/// # Type Parameters
///
/// * `T` - The type to validate
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::validation::{
///     CombinedRule, StringLengthRule, PatternRule, ValidationRule
/// };
///
/// // Create a username validation rule: alphanumeric and 3-20 characters
/// let rule = CombinedRule::new(vec![
///     Box::new(StringLengthRule::new(3, 20)),
///     Box::new(PatternRule::alphanumeric()),
/// ]);
///
/// assert!(rule.validate(&"user123".to_string()).is_valid());
/// assert!(!rule.validate(&"ab".to_string()).is_valid());
/// assert!(!rule.validate(&"user@name".to_string()).is_valid());
/// ```
pub struct CombinedRule<T> {
    /// Rules to combine
    rules: Vec<Box<dyn ValidationRule<T>>>,
}

impl<T> CombinedRule<T> {
    /// Creates a new combined validation rule
    ///
    /// # Arguments
    ///
    /// * `rules` - List of validation rules to combine
    ///
    /// # Returns
    ///
    /// A new combined validation rule
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::validation::{
    ///     CombinedRule, StringLengthRule, PatternRule
    /// };
    ///
    /// // Create password validation rule: 8-64 chars, containing digits
    /// let rule = CombinedRule::new(vec![
    ///     Box::new(StringLengthRule::new(8, 64)),
    ///     Box::new(PatternRule::new("digit", r"\d+"))
    /// ]);
    /// ```
    #[must_use]
    pub fn new(rules: Vec<Box<dyn ValidationRule<T>>>) -> Self {
        Self { rules }
    }
}

impl<T> ValidationRule<T> for CombinedRule<T> {
    fn validate(&self, target: &T) -> ValidationResult {
        let mut result = ValidationResult::Valid;

        for rule in &self.rules {
            result = result.merge(rule.validate(target));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Test our new standard validation rules
    #[test]
    fn test_string_length_rule() {
        let rule = StringLengthRule::new(3, 10);

        // Test valid string
        let result = rule.validate(&"hello".to_string());
        assert!(result.is_valid());

        // Test too short string
        let result = rule.validate(&"hi".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test too long string
        let result = rule.validate(&"this string is too long".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test empty string
        let result = rule.validate(&String::new());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test allow_empty
        let rule = StringLengthRule::new(3, 10).allow_empty();
        let result = rule.validate(&String::new());
        assert!(result.is_valid());
    }

    #[test]
    fn test_pattern_rule() {
        // Test basic pattern rule
        let rule = PatternRule::new("numeric", r"^\d+$");

        let result = rule.validate(&"12345".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"abc123".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test email rule
        let rule = PatternRule::email();

        let result = rule.validate(&"user@example.com".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"invalid-email".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test URL rule
        let rule = PatternRule::url();

        let result = rule.validate(&"https://example.com".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"not a url".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test semver rule
        let rule = PatternRule::semver();

        let result = rule.validate(&"1.2.3".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"1.2.3-beta.1".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"1.2".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test negated rule
        let rule = PatternRule::new("digits", r"\d+").negate();

        let result = rule.validate(&"abcdef".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"abc123".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));
    }

    #[test]
    fn test_numeric_range_rule() {
        // Test integer range
        let rule = NumericRangeRule::new(0, 100);

        let result = rule.validate(&50);
        assert!(result.is_valid());

        let result = rule.validate(&0);
        assert!(result.is_valid());

        let result = rule.validate(&100);
        assert!(result.is_valid());

        let result = rule.validate(&-10);
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        let result = rule.validate(&200);
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test float range
        let rule = NumericRangeRule::new(0.0, 1.0);

        let result = rule.validate(&0.5);
        assert!(result.is_valid());

        let result = rule.validate(&0.0);
        assert!(result.is_valid());

        let result = rule.validate(&1.0);
        assert!(result.is_valid());

        let result = rule.validate(&-0.1);
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        let result = rule.validate(&1.1);
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));
    }

    #[test]
    fn test_parseable_rule() {
        // Test integer parsing
        let rule = ParseableRule::<i32>::new("integer");

        let result = rule.validate(&"42".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"-123".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"not a number".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        let result = rule.validate(&"3.14".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test float parsing
        let rule = ParseableRule::<f64>::new("floating-point number");

        let result = rule.validate(&"3.14".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"42".to_string());
        assert!(result.is_valid());

        let result = rule.validate(&"not a number".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_path_exists_rule() {
        // Test path existence
        let rule = PathExistsRule::exists();

        // Current directory should exist
        let current_dir = std::env::current_dir().unwrap();
        let result = rule.validate(&current_dir);
        assert!(result.is_valid());

        // Non-existent path should fail
        let non_existent = PathBuf::from("/path/that/does/not/exist");
        let result = rule.validate(&non_existent);
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test file rule with a directory
        let rule = PathExistsRule::file();
        let result = rule.validate(&current_dir);
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test directory rule with a directory
        let rule = PathExistsRule::directory();
        let result = rule.validate(&current_dir);
        assert!(result.is_valid());
    }

    #[test]
    fn test_combined_rule() {
        // Create a combined username validation rule
        let rule = CombinedRule::new(vec![
            Box::new(StringLengthRule::new(3, 20)),
            Box::new(PatternRule::alphanumeric()),
        ]);

        // Valid username
        let result = rule.validate(&"user123".to_string());
        assert!(result.is_valid());

        // Too short username
        let result = rule.validate(&"ab".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Non-alphanumeric username
        let result = rule.validate(&"user@name".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Both too short and non-alphanumeric
        let result = rule.validate(&"a@".to_string());
        assert!(!result.is_valid());
        assert!(matches!(result, ValidationResult::Error(_)));
        if let ValidationResult::Error(errors) = result {
            assert_eq!(errors.len(), 2); // Both errors should be included
        }
    }
}
