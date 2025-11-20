# Contribution guidelines

First off, thank you for considering contributing to workspace-tools.

If your contribution is not straightforward, please first discuss the change you
wish to make by creating a new issue before making the change.

## Reporting issues

Before reporting an issue on the
[issue tracker](https://github.com/websublime/workspace-tools/issues),
please check that it has not already been reported by searching for some related
keywords.

## Pull requests

Try to do one pull request per change.

### Merge Strategy

**IMPORTANT:** This project uses **"Rebase and merge"** as the merge strategy. **Do NOT use "Squash and merge"**.

**Why?**
- Our monorepo uses `release-plz` with `git-cliff` to generate separate changelogs for each crate
- Git-cliff filters commits by path using `--include-path` to determine which changes belong to which package
- "Squash and merge" combines all commits into one, making it impossible for git-cliff to correctly attribute changes to specific packages
- This results in empty changelogs for packages that weren't directly modified in the squashed commit
- "Rebase and merge" preserves individual commits, allowing proper changelog generation per package

When merging a PR, always select **"Rebase and merge"** from the merge button dropdown.

### Updating the changelog

Update the changes you have made in
[CHANGELOG](https://github.com/websublime/workspace-tools/blob/main/CHANGELOG.md)
file under the **Unreleased** section.

Add the changes of your pull request to one of the following subsections,
depending on the types of changes defined by
[Keep a changelog](https://keepachangelog.com/en/1.0.0/):

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

If the required subsection does not exist yet under **Unreleased**, create it!

## Developing

### Set up

This is no different than other Rust projects.

```shell
git clone https://github.com/websublime/workspace-tools
cd workspace-tools
cargo test
```

### Useful Commands

- Build and run release version:

  ```shell
  cargo build --release && cargo run --release
  ```

- Run Clippy:

  ```shell
  cargo clippy --all-targets --all-features --workspace
  ```

- Run all tests:

  ```shell
  cargo test --all-features --workspace
  ```

- Check to see if there are code formatting issues

  ```shell
  cargo fmt --all -- --check
  ```

- Format the code in the project

  ```shell
  cargo fmt --all
  ```
