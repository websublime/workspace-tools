//! Graph node trait definition.

/// Trait for nodes in the dependency graph
pub trait Node {
    /// Type representing a dependency relationship
    type DependencyType: std::fmt::Debug + Clone;
    /// Type used to uniquely identify a node
    type Identifier: std::hash::Hash + Eq + Clone + std::fmt::Debug + std::fmt::Display;
    /// Returns a slice of dependencies for this Node
    fn dependencies(&self) -> Vec<&Self::DependencyType>;
    /// Returns dependencies as owned values
    fn dependencies_vec(&self) -> Vec<Self::DependencyType>;
    /// Returns true if the `dependency` can be met by this node
    fn matches(&self, dependency: &Self::DependencyType) -> bool;
    /// Returns the unique identifier for this node
    fn identifier(&self) -> Self::Identifier;
}

/// Wrapper around dependency graph nodes.
/// Differentiates between resolved and unresolved dependencies.
#[derive(Debug, Clone)]
pub enum Step<'a, N: Node> {
    Resolved(&'a N),
    Unresolved(N::DependencyType),
}

impl<'a, N: Node> Step<'a, N> {
    pub fn is_resolved(&self) -> bool {
        matches!(self, Step::Resolved(_))
    }

    pub fn as_resolved(&self) -> Option<&N> {
        match self {
            Step::Resolved(node) => Some(node),
            Step::Unresolved(_) => None,
        }
    }

    pub fn as_unresolved(&self) -> Option<&N::DependencyType> {
        match self {
            Step::Resolved(_) => None,
            Step::Unresolved(dependency) => Some(dependency),
        }
    }
}

impl<'a, N: Node> std::fmt::Display for Step<'a, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Resolved(_) => write!(f, "Resolved"),
            Step::Unresolved(_) => write!(f, "Unresolved"),
        }
    }
}
