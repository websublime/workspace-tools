//! Workspace validation options.
//!
//! This module provides options for customizing validation behavior
//! when checking workspace consistency.

/// Options for validating workspace consistency.
///
/// Customizes how validation is performed, including handling of
/// unresolved dependencies and specification of internal packages.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::ValidationOptions;
///
/// // Default validation options
/// let default_options = ValidationOptions::default();
///
/// // Customized validation options
/// let options = ValidationOptions::new()
///     .treat_unresolved_as_external(true)
///     .with_internal_dependencies(vec![
///         "my-internal-lib",
///         "my-shared-components"
///     ]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ValidationOptions {
    /// Whether to treat unresolved dependencies as external (non-critical).
    pub treat_unresolved_as_external: bool,
    /// List of dependency names to consider as internal (critical if unresolved).
    pub internal_dependencies: Vec<String>,
}

impl ValidationOptions {
    /// Creates new validation options with default settings.
    ///
    /// # Returns
    ///
    /// A new `ValidationOptions` instance with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::ValidationOptions;
    ///
    /// let options = ValidationOptions::new();
    /// ``
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to treat unresolved dependencies as external.
    ///
    /// # Arguments
    ///
    /// * `value` - If `true`, unresolved dependencies are treated as external (not errors)
    ///
    /// # Returns
    ///
    /// The updated `ValidationOptions` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::ValidationOptions;
    ///
    /// let options = ValidationOptions::new()
    ///     .treat_unresolved_as_external(true);
    /// ```
    #[must_use]
    pub fn treat_unresolved_as_external(mut self, value: bool) -> Self {
        self.treat_unresolved_as_external = value;
        self
    }

    /// Sets the list of dependencies to consider internal.
    ///
    /// # Arguments
    ///
    /// * `deps` - Names of dependencies to treat as internal
    ///
    /// # Returns
    ///
    /// The updated `ValidationOptions` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::ValidationOptions;
    ///
    /// let options = ValidationOptions::new()
    ///     .with_internal_dependencies(vec![
    ///         "my-core-lib",
    ///         "my-shared-utils"
    ///     ]);
    /// ```
    #[must_use]
    pub fn with_internal_dependencies<I, S>(mut self, deps: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.internal_dependencies = deps.into_iter().map(Into::into).collect();
        self
    }
}
