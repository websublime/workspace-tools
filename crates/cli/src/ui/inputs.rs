//! Input components for CLI interaction.
//!
//! Provides utilities for user input and interaction.

use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Password, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Configuration for an input prompt
#[allow(clippy::type_complexity)]
pub struct InputPrompt<'a> {
    prompt: String,
    default: Option<String>,
    allow_empty: bool,
    validate_fn: Option<Box<dyn Fn(&str) -> Result<(), String> + 'a>>,
    password: bool,
}

impl<'a> InputPrompt<'a> {
    /// Create a new input prompt
    pub fn new<T: Into<String>>(prompt: T) -> Self {
        InputPrompt {
            prompt: prompt.into(),
            default: None,
            allow_empty: false,
            validate_fn: None,
            password: false,
        }
    }

    /// Set a default value
    pub fn default<T: Into<String>>(mut self, default: T) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Allow empty input
    pub fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    /// Add an input validator
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + 'a,
    {
        self.validate_fn = Some(Box::new(validator));
        self
    }

    /// Make this a password prompt (hidden input)
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Interact with the user and get input
    pub fn interact(&self) -> Result<String, dialoguer::Error> {
        let theme = ColorfulTheme::default();

        if self.password {
            let mut builder = Password::with_theme(&theme).with_prompt(&self.prompt);

            // Password doesn't support default values in dialoguer

            if let Some(validator) = &self.validate_fn {
                builder = builder.validate_with(|input: &String| -> Result<(), String> {
                    if input.is_empty() && !self.allow_empty {
                        return Err("Input cannot be empty".to_string());
                    }
                    validator(input)
                });
            } else if !self.allow_empty {
                builder = builder.validate_with(|input: &String| -> Result<(), String> {
                    if input.is_empty() {
                        return Err("Input cannot be empty".to_string());
                    }
                    Ok(())
                });
            }

            builder.interact()
        } else {
            let mut builder = Input::<String>::with_theme(&theme).with_prompt(&self.prompt);

            if let Some(default) = &self.default {
                builder = builder.default(default.clone());
            }

            if let Some(validator) = &self.validate_fn {
                builder = builder.validate_with(|input: &String| -> Result<(), String> {
                    if input.is_empty() && !self.allow_empty {
                        return Err("Input cannot be empty".to_string());
                    }
                    validator(input)
                });
            } else if !self.allow_empty {
                builder = builder.validate_with(|input: &String| -> Result<(), String> {
                    if input.is_empty() {
                        return Err("Input cannot be empty".to_string());
                    }
                    Ok(())
                });
            }

            builder.interact()
        }
    }
}

/// Get a simple text input
pub fn text<T: Into<String>>(prompt: T) -> Result<String, dialoguer::Error> {
    InputPrompt::new(prompt).interact()
}

/// Get a text input with a default value
pub fn text_with_default<T: Into<String>, D: Into<String> + Clone>(
    prompt: T,
    default: D,
) -> Result<String, dialoguer::Error> {
    InputPrompt::new(prompt).default(default).interact()
}

/// Get a password input (hidden)
pub fn password<T: Into<String>>(prompt: T) -> Result<String, dialoguer::Error> {
    Password::with_theme(&ColorfulTheme::default()).with_prompt(prompt.into()).interact()
}

/// Get confirmation from the user
pub fn confirm<T: Into<String>>(prompt: T, default: bool) -> Result<bool, dialoguer::Error> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.into())
        .default(default)
        .interact()
}

/// Let the user select an option from a list
pub fn select<T: ToString>(prompt: &str, items: &[T]) -> Result<usize, dialoguer::Error> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()
}

/// Let the user select multiple options from a list
pub fn multi_select<T: ToString>(
    prompt: &str,
    items: &[T],
) -> Result<Vec<usize>, dialoguer::Error> {
    MultiSelect::with_theme(&ColorfulTheme::default()).with_prompt(prompt).items(items).interact()
}

/// Create an indeterminate spinner
#[allow(unexpected_cfgs)]
pub fn spinner<T: Into<String>>(message: T) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(message.into());

    // Use a template compatible with the current version of indicatif
    let style = ProgressStyle::default_spinner();
    #[cfg(feature = "fancy")]
    let style = style.tick_strings(&["-", "\\", "|", "/"]);

    let style = style.template("{spinner:.blue} {msg}").unwrap_or_else(|_| {
        // Fallback to a simpler template if parsing fails
        ProgressStyle::default_spinner().template("{msg}").unwrap()
    });

    spinner.set_style(style);
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Create a progress bar
pub fn progress_bar(total: u64) -> ProgressBar {
    let bar = ProgressBar::new(total);
    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap_or_else(|_| {
            // Fallback to a simpler template if parsing fails
            ProgressStyle::default_bar().template("[{elapsed_precise}] {pos}/{len} {msg}").unwrap()
        })
        .progress_chars("##-");

    bar.set_style(style);
    bar
}
