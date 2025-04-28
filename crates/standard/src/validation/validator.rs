use std::collections::HashMap;

/// Result of a validation action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Validation passed successfully
    Valid,
    /// Validation passed with warnings
    Warning(Vec<String>),
    /// Validation failed with errors
    Error(Vec<String>),
}

impl ValidationResult {
    /// Returns true if the validation passed (Valid or Warning)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Error(_))
    }

    /// Returns true if the validation passed without warnings
    #[must_use]
    pub fn is_strictly_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Returns true if the validation has warnings
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        matches!(self, Self::Warning(_))
    }

    /// Returns true if the validation has errors
    #[must_use]
    pub fn has_errors(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Gets the warnings, if any
    #[must_use]
    pub fn warnings(&self) -> Option<&[String]> {
        match self {
            Self::Warning(warnings) => Some(warnings),
            _ => None,
        }
    }

    /// Gets the errors, if any
    #[must_use]
    pub fn errors(&self) -> Option<&[String]> {
        match self {
            Self::Error(errors) => Some(errors),
            _ => None,
        }
    }

    /// Merges two validation results, preferring the more severe result
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            // If either has errors, combine all errors
            (Self::Error(mut errors1), Self::Error(errors2)) => {
                errors1.extend(errors2);
                Self::Error(errors1)
            }
            (Self::Error(errors), Self::Warning(warnings))
            | (Self::Warning(warnings), Self::Error(errors)) => {
                Self::Error([errors, warnings].concat())
            }
            (Self::Error(errors), Self::Valid) | (Self::Valid, Self::Error(errors)) => {
                Self::Error(errors)
            }

            // If no errors but warnings exist, combine all warnings
            (Self::Warning(mut warnings1), Self::Warning(warnings2)) => {
                warnings1.extend(warnings2);
                Self::Warning(warnings1)
            }
            (Self::Warning(warnings), Self::Valid) | (Self::Valid, Self::Warning(warnings)) => {
                Self::Warning(warnings)
            }

            // Both valid
            (Self::Valid, Self::Valid) => Self::Valid,
        }
    }
}

/// Trait for objects that can be validated
pub trait Validatable {
    /// Validates this object
    fn validate(&self) -> ValidationResult;
}

/// A single validation rule
pub trait ValidationRule<T> {
    /// Validates the target object
    fn validate(&self, target: &T) -> ValidationResult;
}

/// A validator for a specific type
pub struct Validator<T> {
    rules: Vec<Box<dyn ValidationRule<T>>>,
}

impl<T> Validator<T> {
    /// Creates a new validator
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a validation rule
    pub fn add_rule<R>(&mut self, rule: R)
    where
        R: ValidationRule<T> + 'static,
    {
        self.rules.push(Box::new(rule));
    }

    /// Validates a target object against all rules
    pub fn validate(&self, target: &T) -> ValidationResult {
        let mut result = ValidationResult::Valid;

        for rule in &self.rules {
            result = result.merge(rule.validate(target));
        }

        result
    }
}

impl<T> Default for Validator<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A validation context for complex validations
#[derive(Debug)]
pub struct ValidationContext {
    /// Additional data for validation
    data: HashMap<String, String>,
    /// Collected warnings during validation
    warnings: Vec<String>,
    /// Collected errors during validation
    errors: Vec<String>,
}

impl ValidationContext {
    /// Creates a new validation context
    #[must_use]
    pub fn new() -> Self {
        Self { data: HashMap::new(), warnings: Vec::new(), errors: Vec::new() }
    }

    /// Adds data to the context
    pub fn add_data(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    /// Gets data from the context
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Adds a warning to the context
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Adds an error to the context
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }

    /// Gets the validation result
    pub fn result(&self) -> ValidationResult {
        if !self.errors.is_empty() {
            ValidationResult::Error(self.errors.clone())
        } else if !self.warnings.is_empty() {
            ValidationResult::Warning(self.warnings.clone())
        } else {
            ValidationResult::Valid
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::expect_used)]
    #[test]
    fn test_validation_result_operations() {
        // Test basic results
        let valid = ValidationResult::Valid;
        let warning = ValidationResult::Warning(vec!["warning".to_string()]);
        let error = ValidationResult::Error(vec!["error".to_string()]);

        assert!(valid.is_valid());
        assert!(valid.is_strictly_valid());
        assert!(!valid.has_warnings());
        assert!(!valid.has_errors());

        assert!(warning.is_valid());
        assert!(!warning.is_strictly_valid());
        assert!(warning.has_warnings());
        assert!(!warning.has_errors());

        assert!(!error.is_valid());
        assert!(!error.is_strictly_valid());
        assert!(!error.has_warnings());
        assert!(error.has_errors());

        let warnings = warning.warnings().expect("Warnings should be present");
        let errors = error.errors().expect("Errors should be present");

        // Test getters
        assert_eq!(warnings[0], "warning");
        assert_eq!(errors[0], "error");
    }

    #[test]
    fn test_validation_result_merge() {
        // Valid + Valid = Valid
        let result = ValidationResult::Valid.merge(ValidationResult::Valid);
        assert!(matches!(result, ValidationResult::Valid));

        // Valid + Warning = Warning
        let result =
            ValidationResult::Valid.merge(ValidationResult::Warning(vec!["warning".to_string()]));
        assert!(matches!(result, ValidationResult::Warning(_)));

        // Warning + Valid = Warning
        let result =
            ValidationResult::Warning(vec!["warning".to_string()]).merge(ValidationResult::Valid);
        assert!(matches!(result, ValidationResult::Warning(_)));

        // Warning + Warning = Warning (combined)
        let result = ValidationResult::Warning(vec!["warning1".to_string()])
            .merge(ValidationResult::Warning(vec!["warning2".to_string()]));
        assert!(matches!(result, ValidationResult::Warning(_)));
        if let ValidationResult::Warning(warnings) = result {
            assert_eq!(warnings.len(), 2);
            assert!(warnings.contains(&"warning1".to_string()));
            assert!(warnings.contains(&"warning2".to_string()));
        }

        // Error + Valid = Error
        let result =
            ValidationResult::Error(vec!["error".to_string()]).merge(ValidationResult::Valid);
        assert!(matches!(result, ValidationResult::Error(_)));

        // Valid + Error = Error
        let result =
            ValidationResult::Valid.merge(ValidationResult::Error(vec!["error".to_string()]));
        assert!(matches!(result, ValidationResult::Error(_)));

        // Error + Warning = Error (contains both)
        let result = ValidationResult::Error(vec!["error".to_string()])
            .merge(ValidationResult::Warning(vec!["warning".to_string()]));
        assert!(matches!(result, ValidationResult::Error(_)));
        if let ValidationResult::Error(errors) = result {
            assert_eq!(errors.len(), 2);
            assert!(errors.contains(&"error".to_string()));
            assert!(errors.contains(&"warning".to_string()));
        }

        // Error + Error = Error (combined)
        let result = ValidationResult::Error(vec!["error1".to_string()])
            .merge(ValidationResult::Error(vec!["error2".to_string()]));
        assert!(matches!(result, ValidationResult::Error(_)));
        if let ValidationResult::Error(errors) = result {
            assert_eq!(errors.len(), 2);
            assert!(errors.contains(&"error1".to_string()));
            assert!(errors.contains(&"error2".to_string()));
        }
    }

    #[test]
    fn test_validator() {
        // Create a simple validation rule
        struct LengthRule {
            min_length: usize,
            max_length: usize,
        }

        impl ValidationRule<String> for LengthRule {
            fn validate(&self, target: &String) -> ValidationResult {
                let length = target.len();
                if length < self.min_length {
                    ValidationResult::Error(vec![format!(
                        "String too short, minimum length is {}",
                        self.min_length
                    )])
                } else if length > self.max_length {
                    ValidationResult::Warning(vec![format!(
                        "String longer than recommended maximum of {}",
                        self.max_length
                    )])
                } else {
                    ValidationResult::Valid
                }
            }
        }

        // Create another validation rule
        struct NoDigitsRule;

        impl ValidationRule<String> for NoDigitsRule {
            fn validate(&self, target: &String) -> ValidationResult {
                if target.chars().any(|c| c.is_ascii_digit()) {
                    ValidationResult::Error(vec!["String should not contain digits".to_string()])
                } else {
                    ValidationResult::Valid
                }
            }
        }

        // Create validator with both rules
        let mut validator = Validator::<String>::new();
        validator.add_rule(LengthRule { min_length: 3, max_length: 10 });
        validator.add_rule(NoDigitsRule);

        // Test valid input
        let result = validator.validate(&"valid".to_string());
        assert!(result.is_strictly_valid());

        // Test too short input
        let result = validator.validate(&"ab".to_string());
        assert!(result.has_errors());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test too long input (warning)
        let result = validator.validate(&"this is too long".to_string());
        assert!(result.has_warnings());
        assert!(matches!(result, ValidationResult::Warning(_)));

        // Test with digits (error)
        let result = validator.validate(&"invalid123".to_string());
        assert!(result.has_errors());
        assert!(matches!(result, ValidationResult::Error(_)));

        // Test combined error and warning
        let result = validator.validate(&"a1".to_string());
        assert!(result.has_errors());
        assert!(matches!(result, ValidationResult::Error(_)));
        if let ValidationResult::Error(errors) = result {
            assert_eq!(errors.len(), 2); // Both too short and contains digits
        }
    }

    #[test]
    fn test_validation_context() {
        let mut context = ValidationContext::new();

        // Add data
        context.add_data("key1", "value1");
        context.add_data("key2", "value2");

        // Check data retrieval
        assert_eq!(context.get_data("key1"), Some(&"value1".to_string()));
        assert_eq!(context.get_data("key2"), Some(&"value2".to_string()));
        assert_eq!(context.get_data("key3"), None);

        // Empty context should be valid
        assert!(matches!(context.result(), ValidationResult::Valid));

        // Add warnings
        context.add_warning("Warning 1");
        context.add_warning("Warning 2");

        // Context with warnings should have warning status
        assert!(matches!(context.result(), ValidationResult::Warning(_)));
        if let ValidationResult::Warning(warnings) = context.result() {
            assert_eq!(warnings.len(), 2);
            assert!(warnings.contains(&"Warning 1".to_string()));
            assert!(warnings.contains(&"Warning 2".to_string()));
        }

        // Add errors
        context.add_error("Error 1");

        // Context with errors should have error status, regardless of warnings
        assert!(matches!(context.result(), ValidationResult::Error(_)));
        if let ValidationResult::Error(errors) = context.result() {
            assert_eq!(errors.len(), 1);
            assert!(errors.contains(&"Error 1".to_string()));
        }
    }

    #[test]
    fn test_validatable_trait() {
        // Create a simple struct that implements Validatable
        struct Person {
            name: String,
            age: i32,
        }

        impl Validatable for Person {
            fn validate(&self) -> ValidationResult {
                let mut context = ValidationContext::new();

                // Check name
                if self.name.is_empty() {
                    context.add_error("Name cannot be empty");
                } else if self.name.len() < 2 {
                    context.add_warning("Name is unusually short");
                }

                // Check age
                if self.age < 0 {
                    context.add_error("Age cannot be negative");
                } else if self.age > 150 {
                    context.add_warning("Age seems unusually high");
                }

                context.result()
            }
        }

        // Test valid person
        let person = Person { name: "Alice".to_string(), age: 30 };
        let result = person.validate();
        assert!(result.is_strictly_valid());

        // Test person with warnings
        let person = Person { name: "A".to_string(), age: 200 };
        let result = person.validate();
        assert!(result.has_warnings());
        assert!(matches!(result, ValidationResult::Warning(_)));
        if let ValidationResult::Warning(warnings) = result {
            assert_eq!(warnings.len(), 2); // Both name and age warnings
        }

        // Test person with errors
        let person = Person { name: String::new(), age: -10 };
        let result = person.validate();
        assert!(result.has_errors());
        assert!(matches!(result, ValidationResult::Error(_)));
        if let ValidationResult::Error(errors) = result {
            assert_eq!(errors.len(), 2); // Both name and age errors
        }
    }
}
