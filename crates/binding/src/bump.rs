//! JavaScript bindings for version bumping functionality.

use crate::types::version::Version;
use napi::Result as NapiResult;
use napi_derive::napi;
use ws_pkg::bump::BumpOptions as WsBumpOptions;

/// JavaScript binding for ws_pkg::bump::BumpOptions
#[napi(object)]
#[derive(Clone)]
pub struct BumpOptions {
    /// Git reference to start from
    pub since: Option<String>,
    /// Explicitly bump to a specific version type
    pub release_as: Option<Version>,
    /// Fetch all branches
    pub fetch_all: Option<bool>,
    /// Fetch tags
    pub fetch_tags: Option<bool>,
    /// Synchronize dependencies
    pub sync_deps: Option<bool>,
    /// Push changes to remote
    pub push: Option<bool>,
}

impl From<BumpOptions> for WsBumpOptions {
    fn from(options: BumpOptions) -> Self {
        Self {
            since: options.since,
            release_as: options.release_as.map(|v| v.into()),
            fetch_all: options.fetch_all,
            fetch_tags: options.fetch_tags,
            sync_deps: options.sync_deps,
            push: options.push,
        }
    }
}

impl From<WsBumpOptions> for BumpOptions {
    fn from(options: WsBumpOptions) -> Self {
        Self {
            since: options.since,
            release_as: options.release_as.map(|v| v.into()),
            fetch_all: options.fetch_all,
            fetch_tags: options.fetch_tags,
            sync_deps: options.sync_deps,
            push: options.push,
        }
    }
}

/// Bump a package version
#[napi(ts_return_type = "string")]
pub fn bump_version(version: String, bump_type: Version) -> NapiResult<String> {
    match bump_type {
        Version::Major => Ok(ws_pkg::types::version::Version::bump_major(&version).to_string()),
        Version::Minor => Ok(ws_pkg::types::version::Version::bump_minor(&version).to_string()),
        Version::Patch => Ok(ws_pkg::types::version::Version::bump_patch(&version).to_string()),
        Version::Snapshot => {
            // For snapshot, we need a SHA, but we'll use a placeholder here
            // In a real implementation, you'd want to pass the SHA as an argument
            let sha = "HEAD";
            Ok(ws_pkg::types::version::Version::bump_snapshot(&version, sha).to_string())
        }
    }
}

/// Bump a package version to a snapshot version with the given SHA
#[napi(ts_return_type = "string")]
pub fn bump_snapshot_version(version: String, sha: String) -> NapiResult<String> {
    Ok(ws_pkg::types::version::Version::bump_snapshot(&version, &sha).to_string())
}

#[cfg(test)]
mod bump_binding_tests {
    use super::*;
    use crate::types::version::Version;

    #[test]
    fn test_bump_options_conversion() {
        // Create BumpOptions
        let options = BumpOptions {
            since: Some("v1.0.0".to_string()),
            release_as: Some(Version::Minor),
            fetch_all: Some(true),
            fetch_tags: Some(true),
            sync_deps: Some(true),
            push: Some(false),
        };

        // Convert to ws_pkg::bump::BumpOptions
        let ws_options = WsBumpOptions::from(options.clone());

        // Verify the conversion
        assert_eq!(ws_options.since, Some("v1.0.0".to_string()));
        assert!(matches!(ws_options.release_as, Some(ws_pkg::types::version::Version::Minor)));
        assert_eq!(ws_options.fetch_all, Some(true));
        assert_eq!(ws_options.fetch_tags, Some(true));
        assert_eq!(ws_options.sync_deps, Some(true));
        assert_eq!(ws_options.push, Some(false));

        // Convert back to BumpOptions
        let options_back = BumpOptions::from(ws_options);

        // Verify the conversion back
        assert_eq!(options_back.since, Some("v1.0.0".to_string()));
        assert!(matches!(options_back.release_as, Some(Version::Minor)));
        assert_eq!(options_back.fetch_all, Some(true));
        assert_eq!(options_back.fetch_tags, Some(true));
        assert_eq!(options_back.sync_deps, Some(true));
        assert_eq!(options_back.push, Some(false));
    }

    #[test]
    fn test_bump_version() {
        // Test major bump
        let result = bump_version("1.0.0".to_string(), Version::Major).unwrap();
        assert_eq!(result, "2.0.0");

        // Test minor bump
        let result = bump_version("1.0.0".to_string(), Version::Minor).unwrap();
        assert_eq!(result, "1.1.0");

        // Test patch bump
        let result = bump_version("1.0.0".to_string(), Version::Patch).unwrap();
        assert_eq!(result, "1.0.1");
    }
}
