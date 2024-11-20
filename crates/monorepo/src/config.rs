use super::changes::ChangesConfig;
use git_cliff_core::config::{
    Bump, ChangelogConfig, CommitParser, Config, GitConfig, RemoteConfig, TextProcessor,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::{Path, PathBuf};
use ws_std::manager::{detect_package_manager, CorePackageManager};
use ws_std::paths::get_project_root_path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolsConfig {
    pub tools: ToolsConfigGroup,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolsConfigGroup {
    pub bump_sync: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    pub package_manager: CorePackageManager,
    pub workspace_root: PathBuf,
    pub changes_config: HashMap<String, String>,
    pub cliff_config: Config,
    pub tools_config: ToolsConfig,
}

#[allow(clippy::too_many_lines)]
fn get_cliff_config(root: &PathBuf) -> Config {
    let default_cliff_config = Config {
        bump: Bump::default(),
        remote: RemoteConfig { ..RemoteConfig::default() },
        changelog: ChangelogConfig {
            header: Some(String::from("# What's Changed")),
            body: Some(String::from(
                r#"
              {%- macro remote_url() -%}
                <REPO>
              {%- endmacro -%}

              {% macro print_commit(commit) -%}
                  - {% if commit.scope %}*({{ commit.scope }})* {% endif %}{% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message | upper_first }} - ([{{ commit.id | truncate(length=7, end="") }}]({{ self::remote_url() }}/commit/{{ commit.id }}))
              {% endmacro -%}

              {% if version %}
                  {% if previous.version %}
                      ## [{{ version | trim_start_matches(pat="v") }}]
                        ({{ self::remote_url() }}/compare/{{ previous.version }}..{{ version }}) - {{ now() | date(format="%Y-%m-%d") }}
                  {% else %}
                      ## [{{ version | trim_start_matches(pat="v") }}] - {{ now() | date(format="%Y-%m-%d") }}
                  {% endif %}
              {% else %}
                  ## [unreleased]
              {% endif %}

              {% for group, commits in commits | group_by(attribute="group") %}
                  ### {{ group | striptags | trim | upper_first }}
                  {% for commit in commits
                  | filter(attribute="scope")
                  | sort(attribute="scope") %}
                      {{ self::print_commit(commit=commit) }}
                  {%- endfor -%}
                  {% raw %}
                  {% endraw %}
                  {%- for commit in commits %}
                      {%- if not commit.scope -%}
                          {{ self::print_commit(commit=commit) }}
                      {% endif -%}
                  {% endfor -%}
              {% endfor %}"#,
            )),
            trim: Some(true),
            postprocessors: Some(vec![TextProcessor {
                pattern: Regex::new("<REPO>").expect("failed to compile regex"),
                replace: Some(String::from("https://github.com/org/repo")),
                replace_command: None,
            }]),
            render_always: Some(false),
            ..ChangelogConfig::default()
        },
        git: GitConfig {
            commit_parsers: Some(vec![
                CommitParser {
                    message: Regex::new("^feat").ok(),
                    group: Some(String::from("<!-- 0 -->⛰️  Features")),
                    ..CommitParser::default()
                },
                CommitParser {
                    message: Regex::new("^fix").ok(),
                    group: Some(String::from("<!-- 1 -->🐛  Bug Fixes")),
                    ..CommitParser::default()
                },
                CommitParser {
                    message: Regex::new("^doc").ok(),
                    group: Some(String::from("<!-- 3 -->📚 Documentation")),
                    ..CommitParser::default()
                },
                CommitParser {
                    message: Regex::new("^perf").ok(),
                    group: Some(String::from("<!-- 4 -->⚡ Performance")),
                    ..CommitParser::default()
                },
                CommitParser {
                    message: Regex::new("^refactor\\(clippy\\)").ok(),
                    skip: Some(true),
                    ..CommitParser::default()
                },
                CommitParser {
                    message: Regex::new("^refactor").ok(),
                    group: Some(String::from("<!-- 2 -->🚜 Refactor")),
                    ..CommitParser::default()
                },
                CommitParser {
                    message: Regex::new("^style").ok(),
                    group: Some(String::from("<!-- 5 -->🎨 Styling")),
                    ..CommitParser::default()
                },
                CommitParser {
                    message: Regex::new("^test").ok(),
                    group: Some(String::from("<!-- 6 -->🧪 Testing")),
                    ..CommitParser::default()
                },
                CommitParser {
                    message: Regex::new("^chore|^ci").ok(),
                    group: Some(String::from("<!-- 7 -->⚙️ Miscellaneous Tasks")),
                    ..CommitParser::default()
                },
                CommitParser {
                    body: Regex::new(".*security").ok(),
                    group: Some(String::from("<!-- 8 -->🛡️ Security")),
                    ..CommitParser::default()
                },
                CommitParser {
                    message: Regex::new("^revert").ok(),
                    group: Some(String::from("<!-- 9 -->◀️ Revert")),
                    ..CommitParser::default()
                },
            ]),
            protect_breaking_commits: Some(false),
            filter_commits: Some(false),
            filter_unconventional: Some(true),
            conventional_commits: Some(true),
            tag_pattern: Regex::new("^((?:@[^/@]+/)?[^/@]+)(?:@([^/]+))?$").ok(),
            skip_tags: Regex::new("beta|alpha|snapshot").ok(),
            ignore_tags: Regex::new("rc|beta|alpha|snapshot").ok(),
            topo_order: Some(false),
            sort_commits: Some(String::from("newest")),
            ..GitConfig::default()
        },
    };

    let root_path = Path::new(root);
    let config_path = &root_path.join(String::from(".config.toml"));

    if config_path.exists() {
        let config_file = File::open(config_path).expect("Failed to open config file");
        let mut config_reader = BufReader::new(config_file);
        let mut buffer = String::new();

        config_reader.read_to_string(&mut buffer).expect("Failed to read confile file");
        let cliff_data = buffer.replace("cliff.", "");

        Config::parse_from_str(cliff_data.as_str()).expect("Failed to parse config content")
    } else {
        default_cliff_config
    }
}

fn get_tools_config(root: &PathBuf) -> ToolsConfig {
    let default_tools_config = ToolsConfig { tools: ToolsConfigGroup { bump_sync: Some(true) } };

    let root_path = Path::new(root);
    let tools_path = &root_path.join(String::from(".config.toml"));

    if tools_path.exists() {
        let config_file = File::open(tools_path).expect("Failed to open config file");
        let mut config_reader = BufReader::new(config_file);
        let mut buffer = String::new();

        config_reader.read_to_string(&mut buffer).expect("Failed to read confile file");

        toml::from_str::<ToolsConfig>(buffer.as_str()).expect("Failed to parse config content")
    } else {
        default_tools_config
    }
}

fn get_changes_config(root: &PathBuf) -> HashMap<String, String> {
    let default_changes_config = HashMap::from([
        ("message".to_string(), "chore(release): |---| release new version".to_string()),
        ("git_user_name".to_string(), "github-actions[bot]".to_string()),
        ("git_user_email".to_string(), "github-actions[bot]@users.noreply.git.com".to_string()),
    ]);

    let root_path = Path::new(root);
    let changes_path = &root_path.join(String::from(".changes.json"));

    if changes_path.exists() {
        let changes_file = File::open(changes_path).expect("Failed to open changes file");
        let changes_reader = BufReader::new(changes_file);

        let changes_config: ChangesConfig =
            serde_json::from_reader(changes_reader).expect("Failed to parse changes file");

        HashMap::from([
            (
                "message".to_string(),
                changes_config.message.expect("Failed to get message from changes file"),
            ),
            (
                "git_user_name".to_string(),
                changes_config
                    .git_user_name
                    .expect("Failed to get git user name from changes file"),
            ),
            (
                "git_user_email".to_string(),
                changes_config
                    .git_user_email
                    .expect("Failed to get git user email from changes file"),
            ),
        ])
    } else {
        default_changes_config
    }
}

#[allow(clippy::needless_pass_by_value)]
fn get_workspace_root(cwd: Option<PathBuf>) -> PathBuf {
    let root = match cwd {
        Some(ref dir) => {
            get_project_root_path(Some(PathBuf::from(dir))).expect("Failed to get project root")
        }
        None => get_project_root_path(None).expect("Failed to get project root"),
    };
    PathBuf::from(&root)
}

pub fn get_workspace_config(cwd: Option<PathBuf>) -> WorkspaceConfig {
    let root = &get_workspace_root(cwd);
    let changes = get_changes_config(root);
    let cliff = get_cliff_config(root);
    let tools = get_tools_config(root);
    let manager = detect_package_manager(root);

    WorkspaceConfig {
        changes_config: changes,
        cliff_config: cliff,
        tools_config: tools,
        workspace_root: root.clone(),
        package_manager: manager.unwrap_or(CorePackageManager::Npm),
    }
}
