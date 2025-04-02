use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use sublime_git_tools::Repo;

/// TestFixture manages a test workspace with a monorepo structure
/// and keeps the TempDir alive for the duration of the test.
pub struct TestFixture {
    // Keep TempDir as first field to ensure it's dropped last
    pub temp_dir: TempDir,
    pub root_path: PathBuf,
    pub repo: Option<Repo>,
    pub packages: Vec<PackageInfo>,
}

/// Information about a package in the monorepo
pub struct PackageInfo {
    pub name: String,
    pub path: PathBuf,
    pub version: String,
    pub dependencies: Vec<(String, String)>,
}

/// Enum representing different package managers
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl TestFixture {
    /// Create a new TestFixture with a basic monorepo structure
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        
        let fixture = Self {
            temp_dir,
            root_path,
            repo: None,
            packages: Vec::new(),
        };
        
        // Create the root package.json
        fixture.create_root_package_json();
        
        // Create .config.toml
        fixture.create_config_file();
        
        // Create .gitattributes
        fixture.create_gitattributes();
        
        fixture
    }
    
    /// Create a new TestFixture with Git initialized
    pub fn with_git() -> Self {
        let mut fixture = Self::new();
        
        // Initialize git repo
        let repo = Repo::create(fixture.root_path.to_str().unwrap())
            .expect("Failed to create git repo");
        
        // Configure git user
        repo.config("sublime-bot", "test-bot@websublime.com")
            .expect("Failed to configure git user");
        
        // Add all files and commit
        repo.add_all()
            .expect("Failed to add files")
            .commit("chore: init monorepo workspace")
            .expect("Failed to commit");
        
        fixture.repo = Some(repo);
        fixture
    }
    
    /// Create a test fixture with the specified package manager
    pub fn with_package_manager(package_manager: PackageManager) -> Self {
        let fixture = Self::with_git();
        
        match package_manager {
            PackageManager::Npm => {
                // Create package-lock.json
                fixture.create_file("package-lock.json", "{}");
            },
            PackageManager::Yarn => {
                // Create yarn.lock
                fixture.create_file("yarn.lock", "");
            },
            PackageManager::Pnpm => {
                // Create pnpm-lock.yaml
                fixture.create_file("pnpm-lock.yaml", "lockfileVersion: '9.0'");
                
                // Create pnpm-workspace.yaml
                fixture.create_file("pnpm-workspace.yaml", "packages:\n - packages/*");
            },
            PackageManager::Bun => {
                // Create bun.lockb (empty file, actual content would be binary)
                fixture.create_file("bun.lockb", "");
            },
        }
        
        // Add and commit package manager files
        if let Some(repo) = &fixture.repo {
            repo.add_all()
                .expect("Failed to add package manager files")
                .commit("chore: add package manager files")
                .expect("Failed to commit package manager files");
        }
        
        fixture
    }
    
    /// Create the package-foo in the monorepo
    pub fn create_package_foo(&mut self) -> &mut Self {
        let package_path = self.root_path.join("packages").join("package-foo");
        fs::create_dir_all(&package_path).expect("Failed to create package-foo directory");
        
        // Create branch if repo exists
        if let Some(repo) = &self.repo {
            repo.create_branch("feature/package-foo").expect("Failed to create branch");
            repo.checkout("feature/package-foo").expect("Failed to checkout branch");
        }
        
        // Create package.json
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
        
        self.create_file(package_path.join("package.json"), package_json);
        
        // Create index.mjs
        let index_mjs = r#"export const foo = "hello foo";"#;
        self.create_file(package_path.join("index.mjs"), index_mjs);
        
        // Add to git and commit if repo exists
        if let Some(repo) = &self.repo {
            repo.add_all()
                .expect("Failed to add package-foo files")
                .commit("feat: add package foo")
                .expect("Failed to commit package-foo files");
                
            // Tag only, don't try to merge as the method doesn't exist
            repo.checkout("main").expect("Failed to checkout main");
            // We would merge here if the method existed
            repo.create_tag("@scope/package-foo@1.0.0", Some("chore: release package-foo@1.0.0".to_string()))
                .expect("Failed to tag package-foo");
        }
        
        // Add package info
        self.packages.push(PackageInfo {
            name: "@scope/package-foo".to_string(),
            path: package_path,
            version: "1.0.0".to_string(),
            dependencies: vec![("@scope/package-bar".to_string(), "1.0.0".to_string())],
        });
        
        self
    }
    
    /// Create the package-bar in the monorepo
    pub fn create_package_bar(&mut self) -> &mut Self {
        let package_path = self.root_path.join("packages").join("package-bar");
        fs::create_dir_all(&package_path).expect("Failed to create package-bar directory");
        
        // Create branch if repo exists
        if let Some(repo) = &self.repo {
            repo.create_branch("feature/package-bar").expect("Failed to create branch");
            repo.checkout("feature/package-bar").expect("Failed to checkout branch");
        }
        
        // Create package.json
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
        
        self.create_file(package_path.join("package.json"), package_json);
        
        // Create index.mjs
        let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;
        self.create_file(package_path.join("index.mjs"), index_mjs);
        
        // Add to git and commit if repo exists
        if let Some(repo) = &self.repo {
            repo.add_all()
                .expect("Failed to add package-bar files")
                .commit("feat: add package bar")
                .expect("Failed to commit package-bar files");
                
            // Tag only, don't try to merge
            repo.checkout("main").expect("Failed to checkout main");
            // We would merge here if the method existed
            repo.create_tag("@scope/package-bar@1.0.0", Some("chore: release package-bar@1.0.0".to_string()))
                .expect("Failed to tag package-bar");
        }
        
        // Add package info
        self.packages.push(PackageInfo {
            name: "@scope/package-bar".to_string(),
            path: package_path,
            version: "1.0.0".to_string(),
            dependencies: vec![("@scope/package-baz".to_string(), "1.0.0".to_string())],
        });
        
        self
    }
    
    /// Create the package-baz in the monorepo
    pub fn create_package_baz(&mut self) -> &mut Self {
        let package_path = self.root_path.join("packages").join("package-baz");
        fs::create_dir_all(&package_path).expect("Failed to create package-baz directory");
        
        // Create branch if repo exists
        if let Some(repo) = &self.repo {
            repo.create_branch("feature/package-baz").expect("Failed to create branch");
            repo.checkout("feature/package-baz").expect("Failed to checkout branch");
        }
        
        // Create package.json
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
        
        self.create_file(package_path.join("package.json"), package_json);
        
        // Create index.mjs
        let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;
        self.create_file(package_path.join("index.mjs"), index_mjs);
        
        // Add to git and commit if repo exists
        if let Some(repo) = &self.repo {
            repo.add_all()
                .expect("Failed to add package-baz files")
                .commit("feat: add package baz")
                .expect("Failed to commit package-baz files");
                
            // Tag only, don't try to merge
            repo.checkout("main").expect("Failed to checkout main");
            // We would merge here if the method existed
            repo.create_tag("@scope/package-baz@1.0.0", Some("chore: release package-baz@1.0.0".to_string()))
                .expect("Failed to tag package-baz");
        }
        
        // Add package info
        self.packages.push(PackageInfo {
            name: "@scope/package-baz".to_string(),
            path: package_path,
            version: "1.0.0".to_string(),
            dependencies: vec![("@scope/package-bar".to_string(), "1.0.0".to_string())],
        });
        
        self
    }
    
    /// Create the package-charlie in the monorepo
    pub fn create_package_charlie(&mut self) -> &mut Self {
        let package_path = self.root_path.join("packages").join("package-charlie");
        fs::create_dir_all(&package_path).expect("Failed to create package-charlie directory");
        
        // Create branch if repo exists
        if let Some(repo) = &self.repo {
            repo.create_branch("feature/package-charlie").expect("Failed to create branch");
            repo.checkout("feature/package-charlie").expect("Failed to checkout branch");
        }
        
        // Create package.json
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
        
        self.create_file(package_path.join("package.json"), package_json);
        
        // Create index.mjs
        let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;
        self.create_file(package_path.join("index.mjs"), index_mjs);
        
        // Add to git and commit if repo exists
        if let Some(repo) = &self.repo {
            repo.add_all()
                .expect("Failed to add package-charlie files")
                .commit("feat: add package charlie")
                .expect("Failed to commit package-charlie files");
                
            // Tag only, don't try to merge
            repo.checkout("main").expect("Failed to checkout main");
            // We would merge here if the method existed
            repo.create_tag("@scope/package-charlie@1.0.0", Some("chore: release package-charlie@1.0.0".to_string()))
                .expect("Failed to tag package-charlie");
        }
        
        // Add package info
        self.packages.push(PackageInfo {
            name: "@scope/package-charlie".to_string(),
            path: package_path,
            version: "1.0.0".to_string(),
            dependencies: vec![("@scope/package-foo".to_string(), "1.0.0".to_string())],
        });
        
        self
    }
    
    /// Create the package-major in the monorepo
    pub fn create_package_major(&mut self) -> &mut Self {
        let package_path = self.root_path.join("packages").join("package-major");
        fs::create_dir_all(&package_path).expect("Failed to create package-major directory");
        
        // Create branch if repo exists
        if let Some(repo) = &self.repo {
            repo.create_branch("feature/package-major").expect("Failed to create branch");
            repo.checkout("feature/package-major").expect("Failed to checkout branch");
        }
        
        // Create package.json
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
        
        self.create_file(package_path.join("package.json"), package_json);
        
        // Create index.mjs
        let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;
        self.create_file(package_path.join("index.mjs"), index_mjs);
        
        // Add to git and commit if repo exists
        if let Some(repo) = &self.repo {
            repo.add_all()
                .expect("Failed to add package-major files")
                .commit("feat: add package major")
                .expect("Failed to commit package-major files");
                
            // Tag only, don't try to merge
            repo.checkout("main").expect("Failed to checkout main");
            // We would merge here if the method existed
            repo.create_tag("@scope/package-major@1.0.0", Some("chore: release package-major@1.0.0".to_string()))
                .expect("Failed to tag package-major");
        }
        
        // Add package info
        self.packages.push(PackageInfo {
            name: "@scope/package-major".to_string(),
            path: package_path,
            version: "1.0.0".to_string(),
            dependencies: vec![
                ("@websublime/pulseio-core".to_string(), "^0.4.0".to_string()),
                ("@websublime/pulseio-style".to_string(), "^1.0.0".to_string()),
                ("lit".to_string(), "^3.0.0".to_string()),
                ("rollup-plugin-postcss-lit".to_string(), "^2.1.0".to_string()),
            ],
        });
        
        self
    }
    
    /// Create the package-tom in the monorepo
    pub fn create_package_tom(&mut self) -> &mut Self {
        let package_path = self.root_path.join("packages").join("package-tom");
        fs::create_dir_all(&package_path).expect("Failed to create package-tom directory");
        
        // Create branch if repo exists
        if let Some(repo) = &self.repo {
            repo.create_branch("feature/package-tom").expect("Failed to create branch");
            repo.checkout("feature/package-tom").expect("Failed to checkout branch");
        }
        
        // Create package.json
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
        
        self.create_file(package_path.join("package.json"), package_json);
        
        // Create index.mjs
        let index_mjs = r#"import { foo } from 'foo';
export const bar = foo + ":from bar";"#;
        self.create_file(package_path.join("index.mjs"), index_mjs);
        
        // Add to git and commit if repo exists
        if let Some(repo) = &self.repo {
            repo.add_all()
                .expect("Failed to add package-tom files")
                .commit("feat: add package tom")
                .expect("Failed to commit package-tom files");
                
            // Tag only, don't try to merge
            repo.checkout("main").expect("Failed to checkout main");
            // We would merge here if the method existed
            repo.create_tag("@scope/package-tom@1.0.0", Some("chore: release package-tom@1.0.0".to_string()))
                .expect("Failed to tag package-tom");
        }
        
        // Add package info
        self.packages.push(PackageInfo {
            name: "@scope/package-tom".to_string(),
            path: package_path,
            version: "1.0.0".to_string(),
            dependencies: vec![
                ("@scope/package-bar".to_string(), "1.0.0".to_string()),
                ("open-props".to_string(), "^1.6.19".to_string()),
                ("postcss".to_string(), "^8.4.35".to_string()),
                ("postcss-cli".to_string(), "^11.0.0".to_string()),
                ("postcss-custom-media".to_string(), "^10.0.3".to_string()),
                ("postcss-import".to_string(), "^16.0.1".to_string()),
                ("postcss-jit-props".to_string(), "^1.0.14".to_string()),
                ("postcss-mixins".to_string(), "^9.0.4".to_string()),
                ("postcss-nested".to_string(), "^6.0.1".to_string()),
                ("postcss-preset-env".to_string(), "^9.4.0".to_string()),
                ("postcss-simple-vars".to_string(), "^7.0.1".to_string()),
                ("typescript".to_string(), "^5.3.3".to_string()),
                ("vite".to_string(), "^5.1.4".to_string()),
            ],
        });
        
        self
    }
    
    /// Create a cycle dependency between packages
    pub fn create_cycle_dependency(&mut self) -> &mut Self {
        // Assuming we have created at least 3 packages
        if self.packages.len() < 3 {
            return self;
        }
        
        // Get the first three packages
        let pkg1 = &self.packages[0];
        let pkg2 = &self.packages[1];
        let pkg3 = &self.packages[2];
        
        // Update package.json files to create a cycle
        // pkg1 depends on pkg2, pkg2 depends on pkg3, pkg3 depends on pkg1
        
        // Update pkg1 package.json
        let pkg1_json_path = pkg1.path.join("package.json");
        let mut pkg1_json = fs::read_to_string(&pkg1_json_path).expect("Failed to read package.json");
        pkg1_json = pkg1_json.replace(
            "\"dependencies\": {",
            &format!("\"dependencies\": {{\n      \"{}\": \"{}\",", pkg2.name, pkg2.version)
        );
        fs::write(&pkg1_json_path, pkg1_json).expect("Failed to write package.json");
        
        // Update pkg2 package.json
        let pkg2_json_path = pkg2.path.join("package.json");
        let mut pkg2_json = fs::read_to_string(&pkg2_json_path).expect("Failed to read package.json");
        pkg2_json = pkg2_json.replace(
            "\"dependencies\": {",
            &format!("\"dependencies\": {{\n      \"{}\": \"{}\",", pkg3.name, pkg3.version)
        );
        fs::write(&pkg2_json_path, pkg2_json).expect("Failed to write package.json");
        
        // Update pkg3 package.json
        let pkg3_json_path = pkg3.path.join("package.json");
        let mut pkg3_json = fs::read_to_string(&pkg3_json_path).expect("Failed to read package.json");
        pkg3_json = pkg3_json.replace(
            "\"dependencies\": {",
            &format!("\"dependencies\": {{\n      \"{}\": \"{}\",", pkg1.name, pkg1.version)
        );
        fs::write(&pkg3_json_path, pkg3_json).expect("Failed to write package.json");
        
        // Add and commit changes if repo exists
        if let Some(repo) = &self.repo {
            repo.add_all()
                .expect("Failed to add cycle dependency changes")
                .commit("feat: create cycle dependency between packages")
                .expect("Failed to commit cycle dependency changes");
        }
        
        self
    }
    
    // Helper methods
    
    /// Create the root package.json file
    fn create_root_package_json(&self) {
        let content = r#"{
"name": "root",
"version": "0.0.0",
"workspaces": [
  "packages/package-foo",
  "packages/package-bar",
  "packages/package-baz",
  "packages/package-charlie",
  "packages/package-major",
  "packages/package-tom"
  ]
}"#;
        
        self.create_file("package.json", content);
    }
    
    /// Create the .config.toml file
    fn create_config_file(&self) {
        let content = r#"[tools]
git_user_name="bot""#;
        
        self.create_file(".config.toml", content);
    }
    
    /// Create the .gitattributes file
    fn create_gitattributes(&self) {
        let content = "* text=auto";
        
        self.create_file(".gitattributes", content);
    }
    
    /// Helper method to create a file with content
    fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) {
        let file_path = self.root_path.join(path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(content.as_bytes()).expect("Failed to write file content");
    }
    
    /// Get a reference to the Git repository
    pub fn get_repo(&self) -> Option<&Repo> {
        self.repo.as_ref()
    }
    
    /// Get the path to the workspace
    pub fn get_path(&self) -> &Path {
        self.root_path.as_path()
    }
    
    /// Create a complete monorepo with all packages
    pub fn create_complete_monorepo() -> Self {
        let mut fixture = Self::with_package_manager(PackageManager::Npm);
        
        fixture
            .create_package_foo()
            .create_package_bar()
            .create_package_baz()
            .create_package_charlie()
            .create_package_major()
            .create_package_tom();
            
        fixture
    }
    
    /// Create a monorepo with a cycle dependency
    pub fn create_monorepo_with_cycle() -> Self {
        let mut fixture = Self::with_package_manager(PackageManager::Npm);
        
        fixture
            .create_package_foo()
            .create_package_bar()
            .create_package_baz()
            .create_cycle_dependency();
            
        fixture
    }
}

/// This is an important wrapper that ensures the TestFixture lives for the duration of the test
pub struct TestContext {
    pub fixture: Arc<TestFixture>,
}

impl TestContext {
    /// Create a new TestContext with a complete monorepo fixture
    pub fn new() -> Self {
        Self {
            fixture: Arc::new(TestFixture::create_complete_monorepo()),
        }
    }
    
    /// Create a new TestContext with a cycle dependency monorepo fixture
    pub fn with_cycle() -> Self {
        Self {
            fixture: Arc::new(TestFixture::create_monorepo_with_cycle()),
        }
    }
    
    /// Create a new TestContext with a specific package manager
    pub fn with_package_manager(package_manager: PackageManager) -> Self {
        Self {
            fixture: Arc::new(TestFixture::with_package_manager(package_manager)),
        }
    }
    
    /// Get the path to the workspace
    pub fn get_path(&self) -> &Path {
        self.fixture.get_path()
    }
}

/// Create a standard test workspace with minimal setup for tests that don't need
/// all the complexity of the TestFixture
pub struct TestWorkspace {
    pub temp_dir: TempDir,
}

#[allow(dead_code)]
impl TestWorkspace {
    pub fn new() -> Self {
        Self { temp_dir: TempDir::new().expect("Failed to create temp dir") }
    }

    pub fn path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    pub fn create_package_json(
        &self,
        relative_path: &str,
        name: &str,
        version: &str,
        deps: &[(&str, &str)],
    ) -> PathBuf {
        let pkg_path = self.temp_dir.path().join(relative_path);
        fs::create_dir_all(&pkg_path).expect("Failed to create package directory");

        let pkg_json_path = pkg_path.join("package.json");
        let mut pkg_json = File::create(&pkg_json_path).expect("Failed to create package.json");

        // Start with basic package.json content
        let mut content = format!(
            r#"{{
  "name": "{name}",
  "version": "{version}",
"#
        );

        // Add dependencies if any
        if !deps.is_empty() {
            content.push_str(
                r#"  "dependencies": {
"#,
            );

            for (i, (dep_name, dep_version)) in deps.iter().enumerate() {
                content.push_str(&format!(r#"    "{dep_name}": "{dep_version}""#));
                if i < deps.len() - 1 {
                    content.push_str(",\n");
                } else {
                    content.push('\n');
                }
            }

            content.push_str("  },\n");
        }

        // Close the JSON
        content.push_str("  \"license\": \"MIT\"\n}");

        pkg_json.write_all(content.as_bytes()).expect("Failed to write package.json");

        pkg_json_path
    }

    pub fn create_monorepo(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Create root package.json
        paths.push(self.create_package_json("", "root-workspace", "1.0.0", &[]));

        // Create some packages
        paths.push(self.create_package_json(
            "packages/pkg-a",
            "pkg-a",
            "1.0.0",
            &[("pkg-c", "^1.0.0")],
        ));

        paths.push(self.create_package_json(
            "packages/pkg-b",
            "pkg-b",
            "2.0.0",
            &[("pkg-a", "^1.0.0"), ("lodash", "^4.0.0")],
        ));

        paths.push(self.create_package_json("packages/pkg-c", "pkg-c", "1.0.0", &[]));

        // Create nested packages
        paths.push(self.create_package_json(
            "apps/web",
            "web-app",
            "0.1.0",
            &[("pkg-a", "^1.0.0"), ("pkg-b", "^2.0.0")],
        ));

        paths
    }

    pub fn create_circular_deps_workspace(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Create root package.json
        paths.push(self.create_package_json("", "circular-workspace", "1.0.0", &[]));

        // Create packages with circular dependencies
        paths.push(self.create_package_json(
            "packages/pkg-a",
            "pkg-a",
            "1.0.0",
            &[("pkg-b", "^1.0.0")],
        ));

        paths.push(self.create_package_json(
            "packages/pkg-b",
            "pkg-b",
            "1.0.0",
            &[("pkg-c", "^1.0.0")],
        ));

        paths.push(self.create_package_json(
            "packages/pkg-c",
            "pkg-c",
            "1.0.0",
            &[("pkg-a", "^1.0.0")], // Circular dependency back to pkg-a
        ));

        paths
    }

    pub fn create_version_conflicts_workspace(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Create root package.json
        paths.push(self.create_package_json("", "conflicts-workspace", "1.0.0", &[]));

        // Create packages with conflicting dependencies
        paths.push(self.create_package_json(
            "packages/pkg-a",
            "pkg-a",
            "1.0.0",
            &[("shared-dep", "^1.0.0")], // Wants shared-dep v1
        ));

        paths.push(self.create_package_json(
            "packages/pkg-b",
            "pkg-b",
            "1.0.0",
            &[("shared-dep", "^2.0.0")], // Wants shared-dep v2
        ));

        paths.push(self.create_package_json("packages/shared-dep", "shared-dep", "2.0.0", &[]));

        paths
    }
}
