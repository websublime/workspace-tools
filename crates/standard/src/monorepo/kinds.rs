//! # Monorepo Kind Implementations
//!
//! ## What
//! This file implements methods for the MonorepoKind enum, providing
//! functionality to identify and work with different types of monorepos.
//!
//! ## How
//! Methods are implemented on the MonorepoKind enum to provide information
//! about each monorepo type, such as its name and configuration file.
//!
//! ## Why
//! Different monorepo systems have different conventions and configuration files.
//! This implementation encapsulates those differences to provide a consistent
//! interface for working with any supported monorepo type.

use super::{types::PackageManagerKind, MonorepoKind};

impl MonorepoKind {
    /// Returns the name of the monorepo kind as a string.
    ///
    /// This is useful for generating human-readable output about the monorepo
    /// type or for configuration purposes.
    ///
    /// # Returns
    ///
    /// A string representing the name of the monorepo kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// let kind = MonorepoKind::YarnWorkspaces;
    /// assert_eq!(kind.name(), "yarn");
    ///
    /// let custom = MonorepoKind::Custom {
    ///     name: "turbo".to_string(),
    ///     config_file: "turbo.json".to_string()
    /// };
    /// assert_eq!(custom.name(), "turbo");
    /// ```
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            MonorepoKind::NpmWorkSpace => String::from("npm"),
            MonorepoKind::YarnWorkspaces => String::from("yarn"),
            MonorepoKind::PnpmWorkspaces => String::from("pnpm"),
            MonorepoKind::BunWorkspaces => String::from("bun"),
            MonorepoKind::DenoWorkspaces => String::from("deno"),
            MonorepoKind::Custom { name, config_file: _ } => name.clone(),
        }
    }

    /// Returns the primary configuration file for this monorepo kind.
    ///
    /// Each monorepo system uses a specific configuration file to define
    /// its structure. This method returns the name of that file.
    ///
    /// # Returns
    ///
    /// A string with the name of the configuration file.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// let kind = MonorepoKind::PnpmWorkspaces;
    /// assert_eq!(kind.config_file(), "pnpm-workspace.yaml");
    /// ```
    pub fn config_file(self) -> String {
        match self {
            MonorepoKind::YarnWorkspaces | MonorepoKind::NpmWorkSpace => {
                String::from("package.json")
            }
            MonorepoKind::PnpmWorkspaces => String::from("pnpm-workspace.yaml"),
            MonorepoKind::BunWorkspaces => String::from("bunfig.toml"),
            MonorepoKind::DenoWorkspaces => String::from("deno.json"),
            MonorepoKind::Custom { name: _, config_file } => config_file.clone(),
        }
    }

    /// Creates a custom monorepo kind with the specified name and config file.
    ///
    /// This method allows dynamically creating custom monorepo definitions
    /// for systems not natively supported by the library.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the custom monorepo system
    /// * `config_file` - The name of the configuration file used by this system
    ///
    /// # Returns
    ///
    /// A new MonorepoKind::Custom variant with the specified properties.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// let npm = MonorepoKind::NpmWorkSpace;
    /// let custom = npm.set_custom("nx".to_string(), "nx.json".to_string());
    ///
    /// assert_eq!(custom.name(), "nx");
    /// assert_eq!(custom.config_file(), "nx.json");
    /// ```
    #[must_use]
    pub fn set_custom(&self, name: String, config_file: String) -> Self {
        MonorepoKind::Custom { name, config_file }
    }
}

impl PackageManagerKind {
    /// Returns the name of the lock file used by this package manager.
    ///
    /// Each package manager uses a specific lock file to record exact versions
    /// of dependencies. This method returns the filename of that lock file.
    ///
    /// # Returns
    ///
    /// A string slice with the name of the lock file.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::types::PackageManagerKind;
    ///
    /// assert_eq!(PackageManagerKind::Npm.lock_file(), "package-lock.json");
    /// assert_eq!(PackageManagerKind::Yarn.lock_file(), "yarn.lock");
    /// assert_eq!(PackageManagerKind::Pnpm.lock_file(), "pnpm-lock.yaml");
    /// ```
    #[must_use]
    pub fn lock_file(self) -> &'static str {
        match self {
            Self::Npm => "package-lock.json", // or npm-shrinkwrap.json
            Self::Yarn => "yarn.lock",
            Self::Pnpm => "pnpm-lock.yaml",
            Self::Bun => "bun.lockb",
            Self::Jsr => "jsr.json",
        }
    }

    /// Returns the command used to invoke this package manager.
    ///
    /// Each package manager has a specific command line tool used to
    /// perform operations like install, update, etc. This method returns
    /// the name of that command.
    ///
    /// # Returns
    ///
    /// A string slice with the command name.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::types::PackageManagerKind;
    ///
    /// assert_eq!(PackageManagerKind::Npm.command(), "npm");
    /// assert_eq!(PackageManagerKind::Yarn.command(), "yarn");
    /// assert_eq!(PackageManagerKind::Pnpm.command(), "pnpm");
    /// ```
    #[must_use]
    pub fn command(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Jsr => "jsr",
        }
    }
}
