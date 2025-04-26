//! Who:
//! Used by developers who need to:
//! - Detect which package manager a project uses
//! - Execute package manager commands
//! - Handle package manager-specific operations
//!
//! Why:
//! Proper package manager handling is essential for:
//! - Reliable dependency management
//! - Correct command execution
//! - Project compatibility

use crate::error::{StandardError, StandardResult};
use std::path::{Path, PathBuf};

/// Supported package manager types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerKind {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManagerKind {
    /// Returns the lock file name for this package manager
    #[must_use]
    pub fn lock_file(self) -> &'static str {
        match self {
            Self::Npm => "package-lock.json", // or npm-shrinkwrap.json
            Self::Yarn => "yarn.lock",
            Self::Pnpm => "pnpm-lock.yaml",
            Self::Bun => "bun.lockb",
        }
    }

    /// Returns the command name for this package manager
    #[must_use]
    pub fn command(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
        }
    }
}

/// Package manager instance
#[derive(Debug)]
pub struct PackageManager {
    kind: PackageManagerKind,
    root: PathBuf,
}

impl PackageManager {
    #[must_use]
    pub fn new(kind: PackageManagerKind, root: impl Into<PathBuf>) -> Self {
        Self { kind, root: root.into() }
    }

    /// Detects the package manager used in the given directory by checking for lock files.
    /// Checks in the order: bun, pnpm, yarn, npm.
    pub fn detect(path: impl AsRef<Path>) -> StandardResult<Self> {
        let path = path.as_ref();

        if path.join(PackageManagerKind::Bun.lock_file()).exists() {
            return Ok(Self::new(PackageManagerKind::Bun, path));
        }
        if path.join(PackageManagerKind::Pnpm.lock_file()).exists() {
            return Ok(Self::new(PackageManagerKind::Pnpm, path));
        }
        if path.join(PackageManagerKind::Yarn.lock_file()).exists() {
            return Ok(Self::new(PackageManagerKind::Yarn, path));
        }
        if path.join(PackageManagerKind::Npm.lock_file()).exists()
            || path.join("npm-shrinkwrap.json").exists()
        // Check both npm lock files
        {
            return Ok(Self::new(PackageManagerKind::Npm, path));
        }

        Err(StandardError::operation(format!(
            "No package manager lock file found in {}",
            path.display()
        )))
    }

    #[must_use]
    pub fn kind(&self) -> PackageManagerKind {
        self.kind
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn lock_file_path(&self) -> PathBuf {
        // Handle npm's alternative lock file if necessary, though detect prioritizes package-lock.json
        let lock_file = if self.kind == PackageManagerKind::Npm
            && !self.root.join(PackageManagerKind::Npm.lock_file()).exists()
            && self.root.join("npm-shrinkwrap.json").exists()
        {
            "npm-shrinkwrap.json"
        } else {
            self.kind.lock_file()
        };
        self.root.join(lock_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_package_manager_kind_files() {
        assert_eq!(PackageManagerKind::Npm.lock_file(), "package-lock.json");
        assert_eq!(PackageManagerKind::Yarn.lock_file(), "yarn.lock");
        assert_eq!(PackageManagerKind::Pnpm.lock_file(), "pnpm-lock.yaml");
        assert_eq!(PackageManagerKind::Bun.lock_file(), "bun.lockb");
    }

    #[test]
    fn test_package_manager_kind_commands() {
        assert_eq!(PackageManagerKind::Npm.command(), "npm");
        assert_eq!(PackageManagerKind::Yarn.command(), "yarn");
        assert_eq!(PackageManagerKind::Pnpm.command(), "pnpm");
        assert_eq!(PackageManagerKind::Bun.command(), "bun");
    }

    #[test]
    fn test_package_manager_detection() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();

        // Test Bun detection (highest priority)
        let bun_lock = root_path.join("bun.lockb");
        let pnpm_lock = root_path.join("pnpm-lock.yaml");
        File::create(&bun_lock).unwrap();
        File::create(&pnpm_lock).unwrap(); // Add lower priority lock file
        let pm_bun = PackageManager::detect(root_path).unwrap();
        assert_eq!(pm_bun.kind(), PackageManagerKind::Bun);
        std::fs::remove_file(&bun_lock).unwrap();
        std::fs::remove_file(&pnpm_lock).unwrap();

        // Test Pnpm detection
        File::create(&pnpm_lock).unwrap();
        let pm_pnpm = PackageManager::detect(root_path).unwrap();
        assert_eq!(pm_pnpm.kind(), PackageManagerKind::Pnpm);
        std::fs::remove_file(&pnpm_lock).unwrap();

        // Test Yarn detection
        let yarn_lock = root_path.join("yarn.lock");
        File::create(&yarn_lock).unwrap();
        let pm_yarn = PackageManager::detect(root_path).unwrap();
        assert_eq!(pm_yarn.kind(), PackageManagerKind::Yarn);
        std::fs::remove_file(&yarn_lock).unwrap();

        // Test npm detection (package-lock.json)
        let npm_lock = root_path.join("package-lock.json");
        File::create(&npm_lock).unwrap();
        let pm_npm = PackageManager::detect(root_path).unwrap();
        assert_eq!(pm_npm.kind(), PackageManagerKind::Npm);
        assert_eq!(pm_npm.root(), root_path);
        assert_eq!(pm_npm.lock_file_path(), npm_lock);
        std::fs::remove_file(&npm_lock).unwrap();

        // Test npm detection (npm-shrinkwrap.json)
        let shrinkwrap = root_path.join("npm-shrinkwrap.json");
        File::create(&shrinkwrap).unwrap();
        let pm_npm_shrinkwrap = PackageManager::detect(root_path).unwrap();
        assert_eq!(pm_npm_shrinkwrap.kind(), PackageManagerKind::Npm);
        assert_eq!(pm_npm_shrinkwrap.lock_file_path(), shrinkwrap); // Should point to shrinkwrap
        std::fs::remove_file(&shrinkwrap).unwrap();
    }

    #[test]
    fn test_no_package_manager() {
        let temp_dir = TempDir::new().unwrap();
        let result = PackageManager::detect(temp_dir.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            StandardError::Operation(msg) => {
                assert!(msg.contains("No package manager lock file found"));
            }
            _ => panic!("Expected Operation error"),
        }
    }
}
