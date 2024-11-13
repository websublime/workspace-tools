use super::command::execute;

use std::{
    env,
    path::{Path, PathBuf},
};

/// Get the project root path.
pub fn get_project_root_path(root: Option<PathBuf>) -> Option<PathBuf> {
    let env_dir = match root {
        Some(dir) => Ok(dir),
        None => env::current_dir(),
    };

    let current_dir = match env_dir {
        Ok(dir) => dir,
        _ => PathBuf::from("./"),
    };
    let current_path = current_dir.as_path();
    let git_root_dir = walk_reverse_dir(current_path);

    let project_root = match git_root_dir {
        Some(current) => current,
        None => {
            let search_root = get_git_root_dir(current_path);
            search_root.unwrap_or(current_path.to_str().unwrap().to_string())
        }
    };

    Some(PathBuf::from(project_root))
}

/// Get the git root directory.
fn get_git_root_dir(dir: &Path) -> Option<String> {
    let top_level =
        execute("git", dir, ["rev-parse", "--show-toplevel"], |stdout, _| Ok(stdout.to_string()));

    match top_level {
        Ok(output) => {
            if output.is_empty() {
                return None;
            }

            Some(output)
        }
        Err(_) => None,
    }
}

/// Walk reverse directory to find the root project.
fn walk_reverse_dir(path: &Path) -> Option<String> {
    let current_path = path.to_path_buf();
    let map_files = vec![
        ("package-lock.json", "npm"),
        ("npm-shrinkwrap.json", "npm"),
        ("yarn.lock", "yarn"),
        ("pnpm-lock.yaml", "pnpm"),
        ("bun.lockb", "bun"),
    ];

    for (file, _) in map_files {
        let lock_file = current_path.join(file);

        if lock_file.exists() {
            return Some(current_path.to_str().unwrap().to_string());
        }
    }

    if let Some(parent) = path.parent() {
        return walk_reverse_dir(parent);
    }

    None
}
