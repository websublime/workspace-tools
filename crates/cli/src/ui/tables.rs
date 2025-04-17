//! Table components for CLI output.
//!
//! Provides simple wrappers for rendering tables with consistent styling.

use std::fmt::Display;
use std::iter::FromIterator;
use tabled::{
    builder::Builder,
    settings::{
        object::Rows,
        style::{BorderColor, LineText, Style},
        themes::ColumnNames,
        Color, Panel,
    },
};

use crate::ui::Symbol;

use super::info_style;

pub struct Tabular {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct TabularOptions {
    pub title: Option<String>,
    pub headers_in_columns: bool,
    pub border_color: Option<Color>,
    pub header_color: Option<Color>,
    pub header_title: Option<String>,
}

pub fn create_tabular(tabular: &Tabular, options: &TabularOptions) -> String {
    let mut rows = tabular.rows.clone();
    rows.insert(0, tabular.headers.clone());

    let mut table = Builder::from_iter(rows).build();

    if let Some(title) = &options.title {
        let text = format!("{} {}", Symbol::info(), info_style(title));
        table.with(LineText::new(text, Rows::first()).offset(2));
    }

    if crate::ui::supports_unicode() {
        table.with(Style::sharp());
    } else {
        table.with(Style::ascii());
    }

    if options.headers_in_columns {
        let mut column = ColumnNames::default();

        if let Some(color) = &options.header_color {
            column = column.color(color.clone());
        }

        table.with(column);
    }

    if let Some(color) = &options.border_color {
        table.with(BorderColor::filled(color.clone()));
    }

    if let Some(title) = &options.header_title {
        let text = format!("{} {}", Symbol::info(), info_style(title));
        table.with(Panel::header(text));
    }

    table.to_string()
}

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
