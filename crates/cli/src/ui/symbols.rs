//! Unicode symbols and status indicators.
//!
//! Provides a consistent set of symbols for status indication
//! with fallbacks for terminals that don't support Unicode.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;

/// Status symbol types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolType {
    Info,
    Success,
    Warning,
    Error,
    Pending,
    Running,
    Bullet,
    ArrowRight,
    Check,
    Cross,
    Star,
    Dot,
}

/// Unicode symbol map
static UNICODE_SYMBOLS: Lazy<HashMap<SymbolType, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(SymbolType::Info, "ℹ");
    m.insert(SymbolType::Success, "✓");
    m.insert(SymbolType::Warning, "⚠");
    m.insert(SymbolType::Error, "✗");
    m.insert(SymbolType::Pending, "⋯");
    m.insert(SymbolType::Running, "⟳");
    m.insert(SymbolType::Bullet, "•");
    m.insert(SymbolType::ArrowRight, "→");
    m.insert(SymbolType::Check, "✓");
    m.insert(SymbolType::Cross, "✗");
    m.insert(SymbolType::Star, "★");
    m.insert(SymbolType::Dot, "·");
    m
});

/// ASCII fallback symbol map
static ASCII_SYMBOLS: Lazy<HashMap<SymbolType, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(SymbolType::Info, "i");
    m.insert(SymbolType::Success, "+");
    m.insert(SymbolType::Warning, "!");
    m.insert(SymbolType::Error, "x");
    m.insert(SymbolType::Pending, "...");
    m.insert(SymbolType::Running, "*");
    m.insert(SymbolType::Bullet, "*");
    m.insert(SymbolType::ArrowRight, "->");
    m.insert(SymbolType::Check, "+");
    m.insert(SymbolType::Cross, "x");
    m.insert(SymbolType::Star, "*");
    m.insert(SymbolType::Dot, ".");
    m
});

/// Status symbol with dynamic rendering based on terminal capabilities
pub struct Symbol {
    symbol_type: SymbolType,
}

impl Symbol {
    /// Create a new symbol of the given type
    pub fn new(symbol_type: SymbolType) -> Self {
        Symbol { symbol_type }
    }

    /// Create an info symbol
    pub fn info() -> Self {
        Symbol::new(SymbolType::Info)
    }

    /// Create a success symbol
    pub fn success() -> Self {
        Symbol::new(SymbolType::Success)
    }

    /// Create a warning symbol
    pub fn warning() -> Self {
        Symbol::new(SymbolType::Warning)
    }

    /// Create an error symbol
    pub fn error() -> Self {
        Symbol::new(SymbolType::Error)
    }

    /// Create a pending symbol
    pub fn pending() -> Self {
        Symbol::new(SymbolType::Pending)
    }

    /// Create a running symbol
    pub fn running() -> Self {
        Symbol::new(SymbolType::Running)
    }

    /// Create a bullet symbol
    pub fn bullet() -> Self {
        Symbol::new(SymbolType::Bullet)
    }

    /// Create an arrow right symbol
    pub fn arrow_right() -> Self {
        Symbol::new(SymbolType::ArrowRight)
    }

    /// Create a check symbol
    pub fn check() -> Self {
        Symbol::new(SymbolType::Check)
    }

    /// Create a cross symbol
    pub fn cross() -> Self {
        Symbol::new(SymbolType::Cross)
    }

    /// Create a star symbol
    pub fn star() -> Self {
        Symbol::new(SymbolType::Star)
    }

    /// Create a dot symbol
    pub fn dot() -> Self {
        Symbol::new(SymbolType::Dot)
    }

    /// Get the raw symbol string
    pub fn as_str(&self) -> &'static str {
        if crate::ui::supports_unicode() {
            UNICODE_SYMBOLS.get(&self.symbol_type).unwrap_or(&"?")
        } else {
            ASCII_SYMBOLS.get(&self.symbol_type).unwrap_or(&"?")
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
