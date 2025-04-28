//! Validation framework for ensuring data integrity.
//!
//! What:
//! This module provides a robust validation framework for validating
//! different types of data against rules and constraints.
//!
//! Who:
//! Used by developers who need to:
//! - Validate input data against rules
//! - Ensure data integrity before processing
//! - Define custom validation rules
//! - Collect validation errors and warnings
//!
//! Why:
//! Proper validation is essential for:
//! - Data integrity and security
//! - Error prevention and detection
//! - Providing useful feedback
//! - Ensuring system reliability

mod rules;
mod validator;

pub use rules::{
    CombinedRule, NumericRangeRule, ParseableRule, PathExistsRule, PatternRule, StringLengthRule,
};
pub use validator::{Validatable, ValidationContext, ValidationResult, ValidationRule, Validator};
