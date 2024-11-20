use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

type ChangesData = BTreeMap<String, ChangeMeta>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Change {
    pub package: String,
    pub release_as: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangeMeta {
    pub deploy: Vec<String>,
    pub pkgs: Vec<Change>,
}

#[derive(Debug, Clone)]
pub struct Changes {
    root: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangesConfig {
    pub message: Option<String>,
    pub git_user_name: Option<String>,
    pub git_user_email: Option<String>,
    pub changes: ChangesData,
}

/*impl From<&WorkspaceConfig> for Changes {
    fn from(config: &WorkspaceConfig) -> Self {
        Changes { root: config.workspace_root.clone() }
    }
}*/

impl From<&PathBuf> for Changes {
    fn from(root: &PathBuf) -> Self {
        Changes { root: root.clone() }
    }
}

impl Changes {
    pub fn new(root: &Path) -> Self {
        Changes { root: root.to_path_buf() }
    }
}
