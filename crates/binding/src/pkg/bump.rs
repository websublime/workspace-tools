use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct BumpOptions {
    pub since: Option<String>,
    pub release_as: Option<String>,
    pub fetch_all: Option<bool>,
    pub fetch_tags: Option<bool>,
    pub sync_deps: Option<bool>,
    pub push: Option<bool>,
}
