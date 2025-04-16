//! Table components for CLI output.
//!
//! Provides simple wrappers for rendering tables with consistent styling.

use std::fmt::Display;
use tabled::{builder::Builder, settings::style::Style};

/// Create a table with modern style
pub fn create_table<D: Display>(headers: Vec<String>, rows: Vec<Vec<D>>) -> String {
    let mut builder = Builder::default();

    // Push headers as first record
    builder.push_record(headers);

    // Push each row as a record
    for row in rows {
        let string_row: Vec<String> = row.into_iter().map(|cell| cell.to_string()).collect();
        builder.push_record(string_row);
    }

    // Build and style the table
    let mut table = builder.build();

    if crate::ui::supports_unicode() {
        table.with(Style::modern());
    } else {
        table.with(Style::ascii());
    }

    table.to_string()
}

/// Convenience function to create a key-value table
pub fn key_value_table<K: Display, V: Display>(items: Vec<(K, V)>) -> String {
    let headers = vec!["Key".to_string(), "Value".to_string()];
    let rows: Vec<Vec<String>> =
        items.into_iter().map(|(k, v)| vec![k.to_string(), v.to_string()]).collect();

    create_table(headers, rows)
}
