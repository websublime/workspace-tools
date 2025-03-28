use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestWorkspace {
    pub temp_dir: TempDir,
}

#[allow(dead_code)]
#[allow(clippy::vec_init_then_push)]
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
