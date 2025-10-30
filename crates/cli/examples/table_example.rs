//! Example demonstrating table rendering capabilities.
//!
//! This example shows how to use the table module to create various
//! types of tables with different themes, alignments, and styling.
//!
//! Run with: cargo run --example table_example

#![allow(clippy::print_stdout)]
#![allow(clippy::too_many_lines)]

use comfy_table::Cell;
use sublime_cli_tools::output::table::{
    ColumnAlignment, TableBuilder, TableTheme, bold_cell, dim_cell, error_cell, info_cell,
    success_cell, truncate_text, warning_cell,
};

fn main() {
    println!("=== Table Rendering Examples ===\n");

    // Example 1: Basic table
    println!("1. Basic Table:");
    let mut table = TableBuilder::new().columns(&["Package", "Version", "Type"]).build();
    table.add_row(&["typescript", "5.3.3", "minor"]);
    table.add_row(&["eslint", "9.0.0", "major"]);
    table.add_row(&["vitest", "1.2.1", "minor"]);
    println!("{}\n", table.render(false));

    // Example 2: Table with different themes
    println!("2. Minimal Theme:");
    let mut table =
        TableBuilder::new().theme(TableTheme::Minimal).columns(&["Name", "Status"]).build();
    table.add_row(&["Build", "Success"]);
    table.add_row(&["Tests", "Passed"]);
    println!("{}\n", table.render(false));

    println!("3. Compact Theme:");
    let mut table =
        TableBuilder::new().theme(TableTheme::Compact).columns(&["ID", "Value"]).build();
    table.add_row(&["1", "100"]);
    table.add_row(&["2", "200"]);
    println!("{}\n", table.render(false));

    // Example 3: Table with custom alignment
    println!("4. Table with Custom Alignment:");
    let mut table = TableBuilder::new()
        .columns(&["Package", "Count", "Status"])
        .alignment(1, ColumnAlignment::Right)
        .alignment(2, ColumnAlignment::Center)
        .build();
    table.add_row(&["@org/core", "42", "✓"]);
    table.add_row(&["@org/utils", "7", "✗"]);
    table.add_row(&["@org/common", "123", "⚠"]);
    println!("{}\n", table.render(false));

    // Example 4: Table with styled cells
    println!("5. Table with Styled Cells:");
    let mut table = TableBuilder::new().columns(&["Status", "Message", "Details"]).build();

    let row1 = vec![success_cell("✓"), Cell::new("Success"), dim_cell("Operation completed")];
    table.add_styled_row(row1);

    let row2 = vec![error_cell("✗"), Cell::new("Failed"), dim_cell("Connection timeout")];
    table.add_styled_row(row2);

    let row3 = vec![warning_cell("⚠"), Cell::new("Warning"), dim_cell("Deprecated option")];
    table.add_styled_row(row3);

    let row4 = vec![info_cell("ℹ"), Cell::new("Info"), dim_cell("Processing 3 files")];
    table.add_styled_row(row4);

    println!("{}\n", table.render(false));

    // Example 5: Table with bold headers and mixed content
    println!("6. Upgrade Report Table:");
    let mut table = TableBuilder::new()
        .columns(&["Package", "Current", "Latest", "Type", "Breaking"])
        .alignment(3, ColumnAlignment::Center)
        .alignment(4, ColumnAlignment::Center)
        .build();

    let row1 = vec![
        Cell::new("typescript"),
        Cell::new("5.0.0"),
        Cell::new("5.3.3"),
        info_cell("minor"),
        Cell::new("No"),
    ];
    table.add_styled_row(row1);

    let row2 = vec![
        Cell::new("eslint"),
        Cell::new("8.0.0"),
        Cell::new("9.0.0"),
        warning_cell("major"),
        error_cell("Yes"),
    ];
    table.add_styled_row(row2);

    let row3 = vec![
        Cell::new("vitest"),
        Cell::new("1.0.0"),
        Cell::new("1.2.1"),
        success_cell("patch"),
        Cell::new("No"),
    ];
    table.add_styled_row(row3);

    println!("{}\n", table.render(false));

    // Example 6: Text truncation
    println!("7. Text Truncation:");
    let long_text = "This is a very long description that needs to be truncated";
    println!("Original: {long_text}");
    println!("Truncated (20): {}", truncate_text(long_text, 20));
    println!("Truncated (30): {}", truncate_text(long_text, 30));
    println!();

    // Example 7: Table with max width
    println!("8. Table with Max Width (60 characters):");
    let mut table = TableBuilder::new().columns(&["Package", "Description"]).max_width(60).build();
    table.add_row(&[
        "@org/workspace-tools",
        "A comprehensive toolkit for managing monorepo workspaces",
    ]);
    table.add_row(&["@org/cli", "Command-line interface for workspace management"]);
    println!("{}\n", table.render(false));

    // Example 8: Empty table
    println!("9. Empty Table:");
    let table = TableBuilder::new().columns(&["Package", "Version"]).build();
    if table.is_empty() {
        println!("Table is empty (no rows: {})\n", table.row_count());
    }

    // Example 9: Plain theme (no borders)
    println!("10. Plain Theme (List Style):");
    let mut table =
        TableBuilder::new().theme(TableTheme::Plain).columns(&["Package", "Version"]).build();
    table.add_row(&["typescript", "5.3.3"]);
    table.add_row(&["eslint", "9.0.0"]);
    table.add_row(&["vitest", "1.2.1"]);
    println!("{}\n", table.render(false));

    // Example 10: Complex data table
    println!("11. Complex Package Information:");
    let mut table = TableBuilder::new()
        .columns(&["Package", "Version", "Files", "Lines", "Status"])
        .alignment(2, ColumnAlignment::Right)
        .alignment(3, ColumnAlignment::Right)
        .alignment(4, ColumnAlignment::Center)
        .build();

    let row1 = vec![
        bold_cell("@myorg/core"),
        Cell::new("1.2.3"),
        Cell::new("42"),
        Cell::new("1,234"),
        success_cell("✓"),
    ];
    table.add_styled_row(row1);

    let row2 = vec![
        bold_cell("@myorg/utils"),
        Cell::new("2.0.1"),
        Cell::new("15"),
        Cell::new("456"),
        success_cell("✓"),
    ];
    table.add_styled_row(row2);

    let row3 = vec![
        bold_cell("@myorg/cli"),
        Cell::new("0.1.0"),
        Cell::new("8"),
        Cell::new("892"),
        warning_cell("⚠"),
    ];
    table.add_styled_row(row3);

    println!("{}\n", table.render(false));

    println!("=== End of Examples ===");
}
