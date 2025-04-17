//! Table components for CLI output.
//!
//! Provides simple wrappers for rendering tables with consistent styling.

use crate::ui::Symbol;
use std::fmt::Display;
use std::iter::FromIterator;
use tabled::{
    builder::Builder,
    settings::{
        object::{Columns, Rows},
        peaker::Priority,
        style::{BorderColor, LineText, Style},
        themes::ColumnNames,
        Border, Color, Height, Panel, Settings, Width,
    },
};
use terminal_size::{terminal_size, Height as TerminalHeight, Width as TerminalWidth};

use super::info_style;

pub struct Tabular {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Default)]
pub struct TabularOptions {
    pub title: Option<String>,
    pub headers_in_columns: bool,
    pub border_color: Option<Color>,
    pub header_color: Option<Color>,
    pub header_title: Option<String>,
    pub footer_title: Option<String>,
}

pub fn create_tabular(tabular: &Tabular, options: &TabularOptions) -> String {
    let (width, height) = get_terminal_size();
    let mut rows = tabular.rows.clone();
    rows.insert(0, tabular.headers.clone());

    let settings = Settings::default()
        .with(Width::wrap(width).priority(Priority::max(true)))
        .with(Width::increase(width))
        .with(Height::limit(height))
        .with(Height::increase(height));

    let mut table = Builder::from_iter(rows).build();
    table
        .with(settings)
        .modify(Columns::single(0), Color::FG_GREEN)
        .modify(Columns::new(1..), Color::FG_BLUE);

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

    if let Some(msg) = &options.footer_title {
        table.with(Panel::footer(msg));
        table.modify(Rows::last(), Border::new().top('─'));
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

pub fn key_value_tabular<K: Display, V: Display>(
    items: Vec<(K, V)>,
    options: Option<TabularOptions>,
) -> String {
    let headers = vec!["Key".to_string(), "Value".to_string()];
    let rows: Vec<Vec<String>> =
        items.into_iter().map(|(k, v)| vec![k.to_string(), v.to_string()]).collect();

    let tabular_options = options.unwrap_or_default();
    let tabular = Tabular { headers, rows };

    create_tabular(&tabular, &tabular_options)
}

fn get_terminal_size() -> (usize, usize) {
    let (TerminalWidth(width), TerminalHeight(height)) =
        terminal_size().expect("failed to obtain a terminal size");

    (width as usize, height as usize)
}
