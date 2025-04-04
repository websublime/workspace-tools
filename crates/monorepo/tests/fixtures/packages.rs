use std::path::Path;
use sublime_git_tools::Repo;

use crate::fixtures::{ensure_dir, write_file};

/// Creates a package in the monorepo
#[allow(clippy::too_many_arguments)]
pub fn create_package(
    repo_path: &Path,
    package_name: &str,
    package_json: &str,
    index_content: &str,
    branch_name: &str,
    commit_message: &str,
    tag: &str,
    tag_message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repo::open(repo_path.to_str().unwrap())?;

    // Create and checkout feature branch
    repo.create_branch(branch_name)?;
    repo.checkout(branch_name)?;

    // Create package directory and files
    let package_path = repo_path.join("packages").join(package_name);
    ensure_dir(&package_path)?;

    write_file(&package_path.join("package.json"), package_json)?;
    write_file(&package_path.join("index.mjs"), index_content)?;

    // Add and commit changes
    repo.add_all()?;
    let _commit_sha = repo.commit(commit_message)?;

    // Checkout main, merge and tag
    repo.checkout("main")?;
    repo.merge(branch_name)?;
    repo.create_tag(tag, Some(tag_message.to_string()))?;

    Ok(())
}

/// Create package-foo in the monorepo
pub fn create_package_foo(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let package_json = r#"{
    "name": "@scope/package-foo",
    "version": "1.0.0",
    "description": "Awesome package foo",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-foo.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-bar": "1.0.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}"#;

    let index_mjs = r#"export const foo = "hello foo";"#;

    create_package(
        repo_path,
        "package-foo",
        package_json,
        index_mjs,
        "feature/package-foo",
        "feat: add package foo",
        "@scope/package-foo@1.0.0",
        "chore: release package-foo@1.0.0",
    )
}

/// Create package-bar in the monorepo
pub fn create_package_bar(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let package_json = r#"{
    "name": "@scope/package-bar",
    "version": "1.0.0",
    "description": "Awesome package bar",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-bar.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-baz": "1.0.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}"#;

    let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;

    create_package(
        repo_path,
        "package-bar",
        package_json,
        index_mjs,
        "feature/package-bar",
        "feat: add package bar",
        "@scope/package-bar@1.0.0",
        "chore: release package-bar@1.0.0",
    )
}

/// Create package-baz in the monorepo
pub fn create_package_baz(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let package_json = r#"{
    "name": "@scope/package-baz",
    "version": "1.0.0",
    "description": "Awesome package baz",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-baz.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-bar": "1.0.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}"#;

    let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;

    create_package(
        repo_path,
        "package-baz",
        package_json,
        index_mjs,
        "feature/package-baz",
        "feat: add package baz",
        "@scope/package-baz@1.0.0",
        "chore: release package-baz@1.0.0",
    )
}

/// Create package-charlie in the monorepo
pub fn create_package_charlie(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let package_json = r#"{
    "name": "@scope/package-charlie",
    "version": "1.0.0",
    "description": "Awesome package charlie",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-charlie.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-foo": "1.0.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}"#;

    let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;

    create_package(
        repo_path,
        "package-charlie",
        package_json,
        index_mjs,
        "feature/package-charlie",
        "feat: add package charlie",
        "@scope/package-charlie@1.0.0",
        "chore: release package-charlie@1.0.0",
    )
}

/// Create package-major in the monorepo
pub fn create_package_major(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let package_json = r#"{
    "name": "@scope/package-major",
    "version": "1.0.0",
    "description": "Awesome package major",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-major.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@websublime/pulseio-core": "^0.4.0",
      "@websublime/pulseio-style": "^1.0.0",
      "lit": "^3.0.0",
      "rollup-plugin-postcss-lit": "^2.1.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}"#;

    let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;

    create_package(
        repo_path,
        "package-major",
        package_json,
        index_mjs,
        "feature/package-major",
        "feat: add package major",
        "@scope/package-major@1.0.0",
        "chore: release package-major@1.0.0",
    )
}

/// Create package-tom in the monorepo
pub fn create_package_tom(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let package_json = r#"{
    "name": "@scope/package-tom",
    "version": "1.0.0",
    "description": "Awesome package tom",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-tom.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-bar": "1.0.0",
      "open-props": "^1.6.19",
      "postcss": "^8.4.35",
      "postcss-cli": "^11.0.0",
      "postcss-custom-media": "^10.0.3",
      "postcss-import": "^16.0.1",
      "postcss-jit-props": "^1.0.14",
      "postcss-mixins": "^9.0.4",
      "postcss-nested": "^6.0.1",
      "postcss-preset-env": "^9.4.0",
      "postcss-simple-vars": "^7.0.1",
      "typescript": "^5.3.3",
      "vite": "^5.1.4"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}"#;

    let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;

    create_package(
        repo_path,
        "package-tom",
        package_json,
        index_mjs,
        "feature/package-tom",
        "feat: add package tom",            // Fixed package name
        "@scope/package-tom@1.0.0",         // Fixed tag
        "chore: release package-tom@1.0.0", // Fixed message
    )
}
