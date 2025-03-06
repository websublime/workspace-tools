//! Version bumping functionality.

use crate::types::version::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BumpOptions {
    pub since: Option<String>,
    pub release_as: Option<Version>,
    pub fetch_all: Option<bool>,
    pub fetch_tags: Option<bool>,
    pub sync_deps: Option<bool>,
    pub push: Option<bool>,
}
