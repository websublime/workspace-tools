# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 0.0.24 - 2026-01-09

### Bug Fixes

#### Changelog

- Use HEAD as fallback when version tag does not exist

<!-- Made with ❤️ by WebSublime -->

## 0.0.23 - 2026-01-09

### Miscellaneous Tasks

<!-- Made with ❤️ by WebSublime -->

## 0.0.22 - 2025-12-12

### Documentation

#### WOR-TSK-198

- Add ExecuteConfig documentation

### Features

#### WOR-TSK-198

- Add ExecuteConfig struct for command execution settings
- Integrate ExecuteConfig into PackageToolsConfig

### Testing

#### WOR-TSK-198

- Add comprehensive tests for ExecuteConfig

<!-- Made with ❤️ by WebSublime -->

## 0.0.21 - 2025-12-12

### Features

#### WOR-TSK-196

- Implement bump integration tests and fix unified prerelease resolution

<!-- Made with ❤️ by WebSublime -->

## 0.0.20 - 2025-12-10

### Documentation

#### WOR-TSK-183

- Fix rustdoc broken intra-doc link in changeset types

<!-- Made with ❤️ by WebSublime -->

## 0.0.19 - 2025-12-05

### Miscellaneous Tasks

<!-- Made with ❤️ by WebSublime -->

## 0.0.18 - 2025-11-27

### Miscellaneous Tasks

<!-- Made with ❤️ by WebSublime -->

## 0.0.17 - 2025-11-25

### Bug Fixes

#### WOR-TSK-168

- Remove include_str for SPEC.md and README.md from lib.rs

### Refactor

#### WOR-TSK-168

- Improve CLI parameter consistency and test organization

<!-- Made with ❤️ by WebSublime -->

## 0.0.16 - 2025-11-25

### Refactor

#### Config

- Decouple config discovery from base crates

<!-- Made with ❤️ by WebSublime -->

## 0.0.15 - 2025-11-21

### Bug Fixes

#### WOR-TSK-166

- Fix duplicate packages and initial commit error in changeset create

<!-- Made with ❤️ by WebSublime -->

## 0.0.14 - 2025-11-21

### Features

#### WOR-TSK-165

- Implement backup control parameter for upgrade apply command

<!-- Made with ❤️ by WebSublime -->

## 0.0.13 - 2025-11-20

### Bug Fixes

#### WOR-TSK-162

- Remove doctests from private prerelease helper methods

### Documentation

#### WOR-TSK-162

- Update pkg crate documentation with prerelease support

### Features

#### WOR-TSK-162

- Implement prerelease parameter support with archive policy

<!-- Made with ❤️ by WebSublime -->

## 0.0.12 - 2025-11-17

### Miscellaneous Tasks

<!-- Made with ❤️ by WebSublime -->

## 0.0.11 - 2025-11-14

### Features

#### WOR-TSK-146

- Implement Story 11.5 - E2E tests for clone command ([#31](https://github.com/websublime/workspace-tools/pull/31))

<!-- Made with ❤️ by WebSublime -->

## 0.0.10 - 2025-11-13

<!-- Made with ❤️ by WebSublime -->

## 0.0.9 - 2025-11-13

<!-- Made with ❤️ by WebSublime -->

## 0.0.8 - 2025-11-13

<!-- Made with ❤️ by WebSublime -->

## 0.0.7 - 2025-11-13

<!-- Made with ❤️ by WebSublime -->

## 0.0.6 - 2025-11-12

### Features

#### WOR-TSK-138

- Add {short_commit} variable support in snapshot format ([#21](https://github.com/websublime/workspace-tools/pull/21))

<!-- Made with ❤️ by WebSublime -->

## 0.0.5 - 2025-11-12

### Bug Fixes

#### WOR-TSK-141

- Refactor git hooks philosophy - pre-commit adds, post-commit validates ([#19](https://github.com/websublime/workspace-tools/pull/19))

<!-- Made with ❤️ by WebSublime -->

## 0.0.4 - 2025-11-11

### Bug Fixes

#### WOR-TSK-140

- Add support for _auth authentication in .npmrc files ([#17](https://github.com/websublime/workspace-tools/pull/17))

<!-- Made with ❤️ by WebSublime -->

## 0.0.3 - 2025-11-11

<!-- Made with ❤️ by WebSublime -->

## 0.0.2 - 2025-11-10

<!-- Made with ❤️ by WebSublime -->

## [0.0.1] - 2025-11-10

### Added

- Initial release of sublime_pkg_tools
- Package management and version control
- Changeset-based versioning system
- Dependency upgrade detection and management
- Project health auditing
