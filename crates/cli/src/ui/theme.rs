//! Color themes and styling constants.
//!
//! Defines color schemes and provides theme management for UI components.

use colored::{Color, Colorize};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// Available themes for UI components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeStyle {
    Default,
    Dark,
    Light,
    NoColor,
}

/// Color palette for theme components
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub primary: Color,
    pub secondary: Color,
    pub muted: Color,
    pub highlight: Color,
    pub dim: Color,
}

/// Current active theme
static CURRENT_THEME: Lazy<RwLock<ThemeStyle>> = Lazy::new(|| RwLock::new(ThemeStyle::Default));

/// Theme palettes lookup
static THEME_PALETTES: Lazy<HashMap<ThemeStyle, ThemePalette>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Default theme
    m.insert(
        ThemeStyle::Default,
        ThemePalette {
            info: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            primary: Color::Blue,
            secondary: Color::Magenta,
            muted: Color::BrightBlack,
            highlight: Color::White,
            dim: Color::BrightBlack,
        },
    );

    // Dark theme
    m.insert(
        ThemeStyle::Dark,
        ThemePalette {
            info: Color::BrightCyan,
            success: Color::BrightGreen,
            warning: Color::BrightYellow,
            error: Color::BrightRed,
            primary: Color::BrightBlue,
            secondary: Color::BrightMagenta,
            muted: Color::BrightBlack,
            highlight: Color::BrightWhite,
            dim: Color::Black,
        },
    );

    // Light theme
    m.insert(
        ThemeStyle::Light,
        ThemePalette {
            info: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            primary: Color::Cyan,
            secondary: Color::Magenta,
            muted: Color::BrightBlack,
            highlight: Color::Black,
            dim: Color::BrightBlack,
        },
    );

    // No color theme (all defaults to none)
    m.insert(
        ThemeStyle::NoColor,
        ThemePalette {
            info: Color::White,
            success: Color::White,
            warning: Color::White,
            error: Color::White,
            primary: Color::White,
            secondary: Color::White,
            muted: Color::White,
            highlight: Color::White,
            dim: Color::White,
        },
    );

    m
});

/// Use the default theme
pub fn use_default_theme() {
    if let Ok(mut theme) = CURRENT_THEME.write() {
        *theme = ThemeStyle::Default;
    }
}

/// Change the active theme
pub fn use_theme(theme_name: &str) -> Result<(), String> {
    let theme = match theme_name.to_lowercase().as_str() {
        "default" => ThemeStyle::Default,
        "dark" => ThemeStyle::Dark,
        "light" => ThemeStyle::Light,
        "nocolor" | "no-color" => ThemeStyle::NoColor,
        _ => return Err(format!("Unknown theme: {}", theme_name)),
    };

    if let Ok(mut current) = CURRENT_THEME.write() {
        *current = theme;
        Ok(())
    } else {
        Err("Failed to acquire write lock on theme".to_string())
    }
}

/// Get the current theme palette
pub fn current_palette() -> ThemePalette {
    if let Ok(theme) = CURRENT_THEME.read() {
        if let Some(palette) = THEME_PALETTES.get(&theme) {
            return *palette;
        }
    }

    // Default fallback if we can't read the theme
    *THEME_PALETTES.get(&ThemeStyle::Default).unwrap()
}

/// Apply the info color to text
pub fn info_style(text: &str) -> colored::ColoredString {
    if !console::colors_enabled() {
        return text.normal();
    }
    text.color(current_palette().info)
}

/// Apply the success color to text
pub fn success_style(text: &str) -> colored::ColoredString {
    if !console::colors_enabled() {
        return text.normal();
    }
    text.color(current_palette().success)
}

/// Apply the warning color to text
pub fn warning_style(text: &str) -> colored::ColoredString {
    if !console::colors_enabled() {
        return text.normal();
    }
    text.color(current_palette().warning)
}

/// Apply the error color to text
pub fn error_style(text: &str) -> colored::ColoredString {
    if !console::colors_enabled() {
        return text.normal();
    }
    text.color(current_palette().error)
}

/// Apply the primary color to text
pub fn primary_style(text: &str) -> colored::ColoredString {
    if !console::colors_enabled() {
        return text.normal();
    }
    text.color(current_palette().primary)
}

/// Apply the secondary color to text
pub fn secondary_style(text: &str) -> colored::ColoredString {
    if !console::colors_enabled() {
        return text.normal();
    }
    text.color(current_palette().secondary)
}

/// Apply the muted color to text
pub fn muted_style(text: &str) -> colored::ColoredString {
    if !console::colors_enabled() {
        return text.normal();
    }
    text.color(current_palette().muted)
}

/// Apply the highlight color to text
pub fn highlight_style(text: &str) -> colored::ColoredString {
    if !console::colors_enabled() {
        return text.normal();
    }
    text.color(current_palette().highlight)
}

/// Apply the dim color to text
pub fn dim_style(text: &str) -> colored::ColoredString {
    if !console::colors_enabled() {
        return text.normal();
    }
    text.color(current_palette().dim)
}
