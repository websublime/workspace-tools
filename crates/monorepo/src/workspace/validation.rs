#[derive(Debug, Clone, Default)]
pub struct ValidationOptions {
    /// Whether to treat unresolved dependencies as external (non-critical).
    pub treat_unresolved_as_external: bool,
    /// List of dependency names to consider as internal (critical if unresolved).
    pub internal_dependencies: Vec<String>,
}

impl ValidationOptions {
    /// Creates new validation options with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to treat unresolved dependencies as external.
    #[must_use]
    pub fn treat_unresolved_as_external(mut self, value: bool) -> Self {
        self.treat_unresolved_as_external = value;
        self
    }

    /// Sets the list of dependencies to consider internal.
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
