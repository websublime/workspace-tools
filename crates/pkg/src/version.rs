use semver::{BuildMetadata, Prerelease, Version as SemVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
/// Enum representing the type of version bump to be performed.
pub enum Version {
    Major,
    Minor,
    Patch,
    Snapshot,
}

impl Version {
    /// Bumps the version of the package to major.
    pub fn bump_major(version: &str) -> SemVersion {
        let mut sem_version = SemVersion::parse(version).unwrap();
        sem_version.major += 1;
        sem_version.minor = 0;
        sem_version.patch = 0;
        sem_version.pre = Prerelease::EMPTY;
        sem_version.build = BuildMetadata::EMPTY;
        sem_version
    }

    /// Bumps the version of the package to minor.
    pub fn bump_minor(version: &str) -> SemVersion {
        let mut sem_version = SemVersion::parse(version).unwrap();
        sem_version.minor += 1;
        sem_version.patch = 0;
        sem_version.pre = Prerelease::EMPTY;
        sem_version.build = BuildMetadata::EMPTY;
        sem_version
    }

    /// Bumps the version of the package to patch.
    pub fn bump_patch(version: &str) -> SemVersion {
        let mut sem_version = SemVersion::parse(version).unwrap();
        sem_version.patch += 1;
        sem_version.pre = Prerelease::EMPTY;
        sem_version.build = BuildMetadata::EMPTY;
        sem_version
    }

    /// Bumps the version of the package to snapshot appending the sha to the version.
    pub fn bump_snapshot(version: &str, sha: &str) -> SemVersion {
        let alpha = format!("alpha.{sha}");

        let mut sem_version = SemVersion::parse(version).unwrap();
        sem_version.pre = Prerelease::new(alpha.as_str()).unwrap_or(Prerelease::EMPTY);
        sem_version.build = BuildMetadata::EMPTY;
        sem_version
    }
}
