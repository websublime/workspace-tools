//! Boxed UI elements for messages and content.
//!
//! Provides functions to create bordered boxes with different styles
//! for various message types.

use super::symbols::Symbol;
use super::theme;
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

/// Box drawing characters
struct BoxChars {
    top_left: &'static str,
    top_right: &'static str,
    bottom_left: &'static str,
    bottom_right: &'static str,
    horizontal: &'static str,
    vertical: &'static str,
}

/// Unicode box drawing characters
const UNICODE_BOX: BoxChars = BoxChars {
    top_left: "┌",
    top_right: "┐",
    bottom_left: "└",
    bottom_right: "┘",
    horizontal: "─",
    vertical: "│",
};

/// ASCII box drawing characters
const ASCII_BOX: BoxChars = BoxChars {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    horizontal: "-",
    vertical: "|",
};

/// Get the appropriate box characters based on terminal capabilities
fn get_box_chars() -> &'static BoxChars {
    if crate::ui::supports_unicode() {
        &UNICODE_BOX
    } else {
        &ASCII_BOX
    }
}

/// Wrap text to fit within max_width, respecting existing line breaks
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut result = Vec::new();

    for line in text.lines() {
        if UnicodeWidthStr::width(line) <= max_width {
            result.push(line.to_string());
            continue;
        }

        let mut current_line = String::new();
        let mut current_width = 0;

        for word in line.split_whitespace() {
            let word_width = UnicodeWidthStr::width(word);

            if current_width + word_width + 1 > max_width {
                if !current_line.is_empty() {
                    result.push(current_line);
                    current_line = String::new();
                    current_width = 0;
                }

                // Handle words that are longer than max_width
                if word_width > max_width {
                    let chars = word.chars();
                    let mut temp_line = String::new();
                    let mut temp_width = 0;

                    for c in chars {
                        let char_width = UnicodeWidthStr::width(c.to_string().as_str());

                        if temp_width + char_width > max_width {
                            result.push(temp_line);
                            temp_line = c.to_string();
                            temp_width = char_width;
                        } else {
                            temp_line.push(c);
                            temp_width += char_width;
                        }
                    }

                    if !temp_line.is_empty() {
                        current_line = temp_line;
                        current_width = temp_width;
                    }
                } else {
                    current_line = word.to_string();
                    current_width = word_width;
                }
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += 1;
                }
                current_line.push_str(word);
                current_width += word_width;
            }
        }

        if !current_line.is_empty() {
            result.push(current_line);
        }
    }

    result
}

/// Create a bordered box with the given title, content, and style
pub fn styled_box(
    title: Option<&str>,
    content: &str,
    style_fn: fn(&str) -> colored::ColoredString,
) -> String {
    let term_width = console::Term::stdout().size().1 as usize;
    let max_width = std::cmp::min(term_width.saturating_sub(4), 100);

    let box_chars = get_box_chars();
    let mut result = String::new();

    // Wrap content to fit within the box
    let wrapped_lines = wrap_text(content, max_width.saturating_sub(4));

    // Calculate box width based on the longest line and title
    let content_width =
        wrapped_lines.iter().map(|line| UnicodeWidthStr::width(line.as_str())).max().unwrap_or(0);

    let title_width = title.map_or(0, |t| UnicodeWidthStr::width(t) + 2);
    let box_width = std::cmp::max(content_width, title_width).max(10);

    // Create top border with optional title
    let top_line = if let Some(title_text) = title {
        let padding = box_width.saturating_sub(UnicodeWidthStr::width(title_text));
        let left_padding = 1;
        let right_padding = padding.saturating_sub(left_padding);

        format!(
            "{}{} {} {}{}",
            style_fn(box_chars.top_left),
            style_fn(&box_chars.horizontal.repeat(left_padding)),
            style_fn(title_text),
            style_fn(&box_chars.horizontal.repeat(right_padding)),
            style_fn(box_chars.top_right)
        )
    } else {
        format!(
            "{}{}{}",
            style_fn(box_chars.top_left),
            style_fn(&box_chars.horizontal.repeat(box_width + 2)),
            style_fn(box_chars.top_right)
        )
    };

    result.push_str(&top_line);
    result.push('\n');

    // Add content lines
    for line in wrapped_lines {
        let padding = box_width.saturating_sub(UnicodeWidthStr::width(line.as_str()));
        result.push_str(&format!(
            "{} {}{} {}",
            style_fn(box_chars.vertical),
            line,
            " ".repeat(padding),
            style_fn(box_chars.vertical)
        ));
        result.push('\n');
    }

    // Add bottom border
    result.push_str(&format!(
        "{}{}{}",
        style_fn(box_chars.bottom_left),
        style_fn(&box_chars.horizontal.repeat(box_width + 2)),
        style_fn(box_chars.bottom_right)
    ));

    result
}

/// Create an info box with the given content
pub fn info_box(content: &str) -> String {
    let symbol = Symbol::info();
    styled_box(Some(&format!(" {} INFO ", symbol)), content, theme::info_style)
}

/// Create a success box with the given content
pub fn success_box(content: &str) -> String {
    let symbol = Symbol::success();
    styled_box(Some(&format!(" {} SUCCESS ", symbol)), content, theme::success_style)
}

/// Create a warning box with the given content
pub fn warning_box(content: &str) -> String {
    let symbol = Symbol::warning();
    styled_box(Some(&format!(" {} WARNING ", symbol)), content, theme::warning_style)
}

/// Create an error box with the given content
pub fn error_box(content: &str) -> String {
    let symbol = Symbol::error();
    styled_box(Some(&format!(" {} ERROR ", symbol)), content, theme::error_style)
}

/// Create a help box with the given content
pub fn help_box(content: &str) -> String {
    styled_box(Some(" HELP "), content, theme::primary_style)
}

/// Create a plain box with no specific styling
pub fn plain_box(title: Option<&str>, content: &str) -> String {
    styled_box(title, content, |s| s.normal())
}
