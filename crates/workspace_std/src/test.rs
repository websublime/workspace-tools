#[cfg(test)]
use crate::git::Repository;

#[cfg(test)]
use crate::manager::CorePackageManager;

#[cfg(test)]
use std::{
    env::temp_dir,
    fs::{create_dir_all, remove_dir_all, File, OpenOptions},
    path::PathBuf,
};

#[cfg(test)]
use std::io::{BufWriter, Write};

#[cfg(test)]
#[cfg(not(windows))]
use std::os::unix::fs::PermissionsExt;

#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MonorepoWorkspace {
    root: PathBuf,
    repository: Repository,
}

#[cfg(test)]
impl MonorepoWorkspace {
    pub fn new() -> Self {
        let temp_dir = temp_dir();
        let monorepo_root_dir = &temp_dir.join("monorepo-workspace");

        if monorepo_root_dir.exists() {
            remove_dir_all(monorepo_root_dir).expect("Unable to remove directory");
        }

        create_dir_all(monorepo_root_dir).expect("Unable to create monorepo directory");

        Self {
            root: monorepo_root_dir.clone(),
            repository: Repository::new(monorepo_root_dir.as_path()),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn create_repository(
        &self,
        package_manager: &CorePackageManager,
    ) -> Result<(), std::io::Error> {
        let monorepo_package_json = &self.root.join("package.json");
        let monorepo_config_toml = &self.root.join(".config.toml");
        let monorepo_packages_dir = &self.root.join("packages");

        create_dir_all(monorepo_packages_dir)?;

        #[cfg(not(windows))]
        std::fs::set_permissions(&self.root, std::fs::Permissions::from_mode(0o777))?;

        let monorepo_root_json = r#"
        {
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

        let package_root_json = serde_json::from_str::<serde_json::Value>(monorepo_root_json)?;
        let monorepo_package_root_json_file = File::create(monorepo_package_json.as_path())?;
        let monorepo_root_json_writer = BufWriter::new(monorepo_package_root_json_file);
        serde_json::to_writer_pretty(monorepo_root_json_writer, &package_root_json)?;

        let monorepo_config_data = r#"
[tools]
bump_sync = true

[cliff.changelog]
# template for the changelog footer
header = """
## What's Changed\n
"""
# template for the changelog body
# https://keats.github.io/tera/docs/#introduction
body = """
{%- macro remote_url() -%}
  <REPO>
{%- endmacro -%}

{% macro print_commit(commit) -%}
    - {% if commit.scope %}*({{ commit.scope }})* {% endif %}\
        {% if commit.breaking %}[**breaking**] {% endif %}\
        {{ commit.message | upper_first }} - \
        ([{{ commit.id | truncate(length=7, end="") }}]({{ self::remote_url() }}/commit/{{ commit.id }}))\
{% endmacro -%}

{% if version %}\
    {% if previous.version %}\
        ## [{{ version | trim_start_matches(pat="v") }}]\
          ({{ self::remote_url() }}/compare/{{ previous.version }}..{{ version }}) - {{ timestamp | date(format="%Y-%m-%d") }}
    {% else %}\
        ## [{{ version | trim_start_matches(pat="v") }}] - {{ now() | date(format="%Y-%m-%d") }}
    {% endif %}\
{% else %}\
    ## [unreleased]
{% endif %}\

{% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | striptags | trim | upper_first }}
    {% for commit in commits
    | filter(attribute="scope")
    | sort(attribute="scope") %}
        {{ self::print_commit(commit=commit) }}
    {%- endfor -%}
    {% raw %}\n{% endraw %}\
    {%- for commit in commits %}
        {%- if not commit.scope -%}
            {{ self::print_commit(commit=commit) }}
        {% endif -%}
    {% endfor -%}
{% endfor %}\n
"""
# template for the changelog footer
footer = """
-- Total Releases: {{ releases | length }} --
"""
# remove the leading and trailing whitespace from the templates
trim = true
# postprocessors
postprocessors = [
  { pattern = '<REPO>', replace = "https://github.com/websublime/workspace-node-tools" }, # replace repository URL
]

[cliff.git]
# parse the commits based on https://www.conventionalcommits.org
conventional_commits = true
# filter out the commits that are not conventional
filter_unconventional = true
# process each line of a commit as an individual commit
split_commits = false
# regex for preprocessing the commit messages
commit_preprocessors = [
  { pattern = '\((\w+\s)?#([0-9]+)\)', replace = "([#${2}](<REPO>/issues/${2}))" },
  # Check spelling of the commit with https://github.com/crate-ci/typos
  # If the spelling is incorrect, it will be automatically fixed.
  { pattern = '.*', replace_command = 'typos --write-changes -' },
]
# regex for parsing and grouping commits
commit_parsers = [
  { message = "^feat", group = "<!-- 0 -->⛰️  Features" },
  { message = "^fix", group = "<!-- 1 -->🐛 Bug Fixes" },
  { message = "^doc", group = "<!-- 3 -->📚 Documentation" },
  { message = "^perf", group = "<!-- 4 -->⚡ Performance" },
  { message = "^refactor\\(clippy\\)", skip = true },
  { message = "^refactor", group = "<!-- 2 -->🚜 Refactor" },
  { message = "^style", group = "<!-- 5 -->🎨 Styling" },
  { message = "^test", group = "<!-- 6 -->🧪 Testing" },
  { message = "^chore\\(release\\): prepare for", skip = true },
  { message = "^chore\\(deps.*\\)", skip = true },
  { message = "^chore\\(pr\\)", skip = true },
  { message = "^chore\\(pull\\)", skip = true },
  { message = "^chore\\(npm\\).*yarn\\.lock", skip = true },
  { message = "^chore|^ci", group = "<!-- 7 -->⚙️ Miscellaneous Tasks" },
  { body = ".*security", group = "<!-- 8 -->🛡️ Security" },
  { message = "^revert", group = "<!-- 9 -->◀️ Revert" },
]
# protect breaking changes from being skipped due to matching a skipping commit_parser
protect_breaking_commits = false
# filter out the commits that are not matched by commit parsers
filter_commits = false
# regex for matching git tags
tag_pattern = "v[0-9].*"
# regex for skipping tags
skip_tags = "beta|alpha"
# regex for ignoring tags
ignore_tags = "rc|v2.1.0|v2.1.1"
# sort the tags topologically
topo_order = false
# sort the commits inside sections by oldest/newest order
sort_commits = "newest"
"#;

        let mut monorepo_package_config_toml_file = File::create(monorepo_config_toml.as_path())?;
        monorepo_package_config_toml_file.write_all(monorepo_config_data.as_bytes())?;

        match package_manager {
            CorePackageManager::Yarn => {
                let yarn_lock = &self.root.join("yarn.lock");
                File::create(yarn_lock)?;
            }
            CorePackageManager::Pnpm => {
                let pnpm_lock = &self.root.join("pnpm-lock.yaml");
                let pnpm_workspace = &self.root.join("pnpm-workspace.yaml");

                let mut lock_file = File::create(pnpm_lock)?;
                lock_file.write_all(r"lockfileVersion: '9.0'".as_bytes())?;

                let mut workspace_file = File::create(pnpm_workspace)?;
                workspace_file.write_all(
                    r#"
                    packages:
                      - "packages/*"
                "#
                    .as_bytes(),
                )?;
            }
            CorePackageManager::Bun => {
                let bun_lock = &self.root.join("bun.lockb");
                File::create(bun_lock)?;
            }
            CorePackageManager::Npm => {
                let npm_lock = &self.root.join("package-lock.json");
                File::create(npm_lock)?;
            }
        }

        self.repository
            .init("main", "Websublime Machine", "machine@websublime.com")
            .expect("Failed to initialize git repository");
        self.repository.add_all().expect("Failed to add all files");
        self.repository
            .commit("chore: init monorepo workspace", None, None)
            .expect("Failed to commit changes");

        Ok(())
    }

    pub fn create_changes(&self) -> Result<(), std::io::Error> {
        let monorepo_changes_json = &self.root.join(".changes.json");

        self.repository
            .create_branch("feature/changes")
            .expect("Failed to create branch feature/changes");

        let monorepo_changes_json_data = r#"
      {
          "message": "chore(release): release new version",
          "git_user_name": "github-actions[bot]",
          "git_user_email": "github-actions[bot]@users.noreply.git.com",
          "changes": {}
      }"#;

        let package_changes_json =
            serde_json::from_str::<serde_json::Value>(monorepo_changes_json_data)?;
        let monorepo_package_changes_json_file = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(monorepo_changes_json.as_path())?;
        let monorepo_changes_json_writer = BufWriter::new(monorepo_package_changes_json_file);
        serde_json::to_writer_pretty(monorepo_changes_json_writer, &package_changes_json)?;

        self.repository.add_all().expect("Failed to add all files");
        self.repository
            .commit("feat: add changes file", None, None)
            .expect("Failed to commit changes");
        self.repository.checkout("main").expect("Failed to checkout main branch");
        self.repository.merge("feature/changes").expect("Failed to merge branches");

        Ok(())
    }

    pub fn create_package_foo(&self) -> Result<(), std::io::Error> {
        self.repository
            .create_branch("feature/package-foo")
            .expect("Failed to create branch feature/package-foo");

        let monorepo_packages_dir = &self.root.join("packages");
        let monorepo_package_foo_dir = &monorepo_packages_dir.join("package-foo");
        let js_path = &monorepo_package_foo_dir.join("index.mjs");

        create_dir_all(monorepo_package_foo_dir)?;

        let package_foo_json = r#"
      {
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
        let package_foo_json = serde_json::from_str::<serde_json::Value>(package_foo_json)?;
        let monorepo_package_foo_json_file = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(monorepo_package_foo_dir.join("package.json").as_path())?;
        let monorepo_package_foo_json_writer = BufWriter::new(monorepo_package_foo_json_file);
        serde_json::to_writer_pretty(monorepo_package_foo_json_writer, &package_foo_json)?;

        let mut js_file = File::create(js_path)?;
        js_file.write_all(r#"export const foo = "hello foo";"#.as_bytes())?;

        self.repository.add_all().expect("Failed to add all files");
        self.repository
            .commit("feat: add package foo", None, None)
            .expect("Failed to commit package foo");
        self.repository.checkout("main").expect("Failed to checkout main branch");
        self.repository.merge("feature/package-foo").expect("Failed to merge branches");
        self.repository
            .tag("@scope/package-foo@1.0.0", Some("chore: release package-foo@1.0.0".to_string()))
            .expect("Failed to create tag");

        Ok(())
    }

    #[allow(clippy::items_after_statements)]
    pub fn create_package_bar(&self) -> Result<(), std::io::Error> {
        self.repository
            .create_branch("feature/package-bar")
            .expect("Failet to create branch feature/package-bar");

        let monorepo_packages_dir = &self.root.join("packages");
        let monorepo_package_bar_dir = &monorepo_packages_dir.join("package-bar");
        let js_path = &monorepo_package_bar_dir.join("index.mjs");

        create_dir_all(monorepo_package_bar_dir)?;

        let package_bar_json = r#"
      {
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
        let package_bar_json = serde_json::from_str::<serde_json::Value>(package_bar_json)?;
        let monorepo_package_bar_json_file =
            File::create(monorepo_package_bar_dir.join("package.json").as_path())?;
        let monorepo_package_bar_json_writer = BufWriter::new(monorepo_package_bar_json_file);
        serde_json::to_writer_pretty(monorepo_package_bar_json_writer, &package_bar_json)?;

        #[cfg(windows)]
        const LINE_ENDING: &str = "\r\n";
        #[cfg(not(windows))]
        const LINE_ENDING: &str = "\n";

        let mut js_file = File::create(js_path.as_path())?;
        js_file.write_all(format!(r#"export const bar = "hello bar";{LINE_ENDING}"#).as_bytes())?;

        self.repository
            .add(monorepo_package_bar_dir.join("package.json").as_path())
            .expect("Failed to add all files");
        self.repository.add(js_path.as_path()).expect("Failed to add all files");
        self.repository
            .commit("feat: add package bar", None, None)
            .expect("Failed to commit changes");

        self.repository.checkout("main").expect("Failed to checkout main branch");
        self.repository.merge("feature/package-bar").expect("Failed to merge branches");
        self.repository
            .tag("@scope/package-bar@1.0.0", Some("chore: release package-bar@1.0.0".to_string()))
            .expect("Failed to create tag");

        Ok(())
    }

    pub fn create_package_baz(&self) -> Result<(), std::io::Error> {
        self.repository
            .create_branch("feature/package-baz")
            .expect("Failet to create branch feature/package-baz");

        let monorepo_packages_dir = &self.root.join("packages");
        let monorepo_package_baz_dir = &monorepo_packages_dir.join("package-baz");
        let js_path = &monorepo_package_baz_dir.join("index.mjs");

        create_dir_all(monorepo_package_baz_dir)?;

        let package_baz_json = r#"
      {
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
        let package_baz_json = serde_json::from_str::<serde_json::Value>(package_baz_json)?;
        let monorepo_package_baz_json_file = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(monorepo_package_baz_dir.join("package.json").as_path())?;
        let monorepo_package_baz_json_writer = BufWriter::new(monorepo_package_baz_json_file);
        serde_json::to_writer_pretty(monorepo_package_baz_json_writer, &package_baz_json)?;

        let mut js_file = File::create(js_path)?;
        js_file.write_all(r#"export const baz = "hello baz";"#.as_bytes())?;

        self.repository.add_all().expect("Failed to add all files");
        self.repository
            .commit("feat: add package baz", None, None)
            .expect("Failed to commit changes");
        self.repository.checkout("main").expect("Failed to checkout main branch");
        self.repository.merge("feature/package-baz").expect("Failed to merge branches");
        self.repository
            .tag("@scope/package-baz@1.0.0", Some("chore: release package-baz@1.0.0".to_string()))
            .expect("Failed to create tag");

        Ok(())
    }

    pub fn create_package_charlie(&self) -> Result<(), std::io::Error> {
        self.repository
            .create_branch("feature/package-charlie")
            .expect("Failet to create branch feature/package-charlie");

        let monorepo_packages_dir = &self.root.join("packages");
        let monorepo_package_charlie_dir = &monorepo_packages_dir.join("package-charlie");
        let js_path = &monorepo_package_charlie_dir.join("index.mjs");

        create_dir_all(monorepo_package_charlie_dir)?;

        let package_charlie_json = r#"
      {
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
        let package_charlie_json = serde_json::from_str::<serde_json::Value>(package_charlie_json)?;
        let monorepo_package_charlie_json_file = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(monorepo_package_charlie_dir.join("package.json").as_path())?;
        let monorepo_package_charlie_json_writer =
            BufWriter::new(monorepo_package_charlie_json_file);
        serde_json::to_writer_pretty(monorepo_package_charlie_json_writer, &package_charlie_json)?;

        let mut js_file = File::create(js_path)?;
        js_file.write_all(r#"export const charlie = "hello charlie";"#.as_bytes())?;

        self.repository.add_all().expect("Failed to add all files");
        self.repository
            .commit("feat: add package charlie", None, None)
            .expect("Failed to commit changes");
        self.repository.checkout("main").expect("Failed to checkout main branch");
        self.repository.merge("feature/package-charlie").expect("Failed to merge branches");
        self.repository
            .tag(
                "@scope/package-charlie@1.0.0",
                Some("chore: release package-charlie@1.0.0".to_string()),
            )
            .expect("Failed to create tag");

        Ok(())
    }

    pub fn create_package_major(&self) -> Result<(), std::io::Error> {
        self.repository
            .create_branch("feature/package-major")
            .expect("Failet to create branch feature/package-major");

        let monorepo_packages_dir = &self.root.join("packages");
        let monorepo_package_major_dir = &monorepo_packages_dir.join("package-major");
        let js_path = &monorepo_package_major_dir.join("index.mjs");

        create_dir_all(monorepo_package_major_dir)?;

        let package_major_json = r#"
      {
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
        let package_major_json = serde_json::from_str::<serde_json::Value>(package_major_json)?;
        let monorepo_package_major_json_file = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(monorepo_package_major_dir.join("package.json").as_path())?;
        let monorepo_package_major_json_writer = BufWriter::new(monorepo_package_major_json_file);
        serde_json::to_writer_pretty(monorepo_package_major_json_writer, &package_major_json)?;

        let mut js_file = File::create(js_path)?;
        js_file.write_all(r#"export const major = "hello major";"#.as_bytes())?;

        self.repository.add_all().expect("Failed to add all files");
        self.repository
            .commit("feat: add package major", None, None)
            .expect("Failed to commit changes");
        self.repository.checkout("main").expect("Failed to checkout main branch");
        self.repository.merge("feature/package-major").expect("Failed to merge branches");
        self.repository
            .tag(
                "@scope/package-major@1.0.0",
                Some("chore: release package-major@1.0.0".to_string()),
            )
            .expect("Failed to create tag");

        Ok(())
    }

    pub fn create_package_tom(&self) -> Result<(), std::io::Error> {
        self.repository
            .create_branch("feature/package-tom")
            .expect("Failet to create branch feature/package-tom");

        let monorepo_packages_dir = &self.root.join("packages");
        let monorepo_package_tom_dir = &monorepo_packages_dir.join("package-tom");
        let js_path = &monorepo_package_tom_dir.join("index.mjs");

        create_dir_all(monorepo_package_tom_dir)?;

        let package_tom_json = r#"
      {
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
        let package_tom_json = serde_json::from_str::<serde_json::Value>(package_tom_json)?;
        let monorepo_package_tom_json_file = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(monorepo_package_tom_dir.join("package.json").as_path())?;
        let monorepo_package_tom_json_writer = BufWriter::new(monorepo_package_tom_json_file);
        serde_json::to_writer_pretty(monorepo_package_tom_json_writer, &package_tom_json)?;

        let mut js_file = File::create(js_path)?;
        js_file.write_all(r#"export const tom = "hello tom";"#.as_bytes())?;

        self.repository.add_all().expect("Failed to add all files");
        self.repository
            .commit("feat: add package tom", None, None)
            .expect("Failed to commit changes");
        self.repository.checkout("main").expect("Failed to checkout main branch");
        self.repository.merge("feature/package-tom").expect("Failed to merge branches");
        self.repository
            .tag("@scope/package-tom@1.0.0", Some("chore: release package-tom@1.0.0".to_string()))
            .expect("Failed to create tag");

        Ok(())
    }

    pub fn create_workspace(
        &self,
        package_manager: &CorePackageManager,
    ) -> Result<(), std::io::Error> {
        self.create_repository(package_manager)?;
        self.create_changes()?;
        self.create_package_bar()?;
        self.create_package_foo()?;
        self.create_package_baz()?;
        self.create_package_charlie()?;
        self.create_package_major()?;
        self.create_package_tom()?;
        Ok(())
    }

    pub fn delete_repository(&self) -> bool {
        remove_dir_all(&self.root).is_ok()
    }

    pub fn get_monorepo_root(&self) -> &PathBuf {
        &self.root
    }
}

#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
mod tests {
    use super::*;

    #[test]
    fn test_create_repository() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        monorepo.create_repository(&CorePackageManager::Npm)?;

        let status = monorepo.repository.status().expect("Failed to get status");
        let uncleaned = monorepo.repository.is_workdir_unclean().expect("Workdir is not clean");
        let log = monorepo.repository.log().expect("Failed to get log");
        let local = monorepo.repository.list_config("local").expect("Failed to get local config");
        dbg!(&status);
        dbg!(&uncleaned);
        dbg!(&log);
        dbg!(&local);
        dbg!(&monorepo.root);

        assert!(monorepo.get_monorepo_root().exists());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_create_changes() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        monorepo.create_repository(&CorePackageManager::Npm)?;
        monorepo.create_changes()?;

        let status = monorepo.repository.status().expect("Failed to get status");
        let uncleaned = monorepo.repository.is_workdir_unclean().expect("Workdir is not clean");
        let log = monorepo.repository.log().expect("Failed to get log");
        let local = monorepo.repository.list_config("local").expect("Failed to get local config");
        dbg!(&status);
        dbg!(&uncleaned);
        dbg!(&log);
        dbg!(&local);
        dbg!(&monorepo.root);

        assert!(monorepo.get_monorepo_root().exists());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_create_package_bar() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        monorepo.create_repository(&CorePackageManager::Npm)?;
        monorepo.create_changes()?;
        monorepo.create_package_bar()?;

        let status = monorepo.repository.status().expect("Failed to get status");
        let uncleaned = monorepo.repository.is_workdir_unclean().expect("Workdir is not clean");
        let log = monorepo.repository.log().expect("Failed to get log");
        let local = monorepo.repository.list_config("local").expect("Failed to get local config");
        dbg!(&status);
        dbg!(&uncleaned);
        dbg!(&log);
        dbg!(&local);
        dbg!(&monorepo.root);

        assert!(monorepo.get_monorepo_root().exists());

        monorepo.delete_repository();

        Ok(())
    }
}
