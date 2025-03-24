#[cfg(test)]
mod repo_tests {
    use std::{
        env::temp_dir,
        fs::{canonicalize, create_dir, remove_dir_all, File},
        io::Write,
        path::PathBuf,
    };
    use sublime_git_tools::{GitFileStatus, Repo, RepoError};
    use sublime_standard_tools::get_project_root_path;

    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(not(windows))]
    use std::fs::set_permissions;

    fn create_workspace() -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let monorepo_root_dir = temp_dir.join("monorepo-workspace");

        if monorepo_root_dir.exists() {
            remove_dir_all(&monorepo_root_dir)?;
        }

        create_dir(&monorepo_root_dir)?;

        let mut readme_file = File::create(monorepo_root_dir.join("package-lock.json").as_path())?;
        readme_file.write_all(b"{}")?;

        #[cfg(not(windows))]
        set_permissions(&monorepo_root_dir, std::fs::Permissions::from_mode(0o777))?;

        let root = canonicalize(monorepo_root_dir.as_path()).expect("Failed to canonicalize path");

        Ok(root)
    }

    #[test]
    fn test_repo_open() -> Result<(), RepoError> {
        let current_dir = std::env::current_dir().unwrap();
        let project_root = get_project_root_path(Some(current_dir)).unwrap();

        let repo = Repo::open(project_root.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Compare canonical paths to handle different string formats
        let repo_path = std::fs::canonicalize(repo.get_repo_path()).unwrap();
        let expected_path = std::fs::canonicalize(&project_root).unwrap();

        assert_eq!(repo_path, expected_path);

        Ok(())
    }

    #[test]
    fn test_create_branch() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;
        let result = repo.create_branch("feat/my-new-feature");

        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_list_branches() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;
        repo.create_branch("feat/my-new-feature")?;
        let branches = repo.list_branches()?;

        // Check if branches contain main and feat/my-new-feature
        assert!(branches.contains(&String::from("main")));
        assert!(branches.contains(&String::from("feat/my-new-feature")));

        Ok(())
    }

    #[test]
    fn test_list_config() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        let config = repo.config("Sublime Git Bot", "git-boot@websublime.com")?.list_config()?;

        assert!(!config.is_empty());

        Ok(())
    }

    #[test]
    fn test_checkout_branch() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;
        repo.create_branch("feat/my-new-feature")?;
        repo.checkout("feat/my-new-feature")?;

        let current_branch = repo.get_current_branch()?;

        assert_eq!(current_branch, String::from("feat/my-new-feature"));

        Ok(())
    }

    #[test]
    fn test_get_current_branch() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;
        let current_branch = repo.get_current_branch()?;

        assert_eq!(current_branch, String::from("main"));

        Ok(())
    }

    #[test]
    fn test_get_last_tag() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;
        repo.create_tag("v1.0.0", None)?;
        repo.create_tag("v1.1.0", Some("chore: tag".to_string()))?;
        let last_tag = repo.get_last_tag()?;

        assert_eq!(last_tag, String::from("v1.1.0"));

        Ok(())
    }

    #[test]
    fn test_get_current_sha() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;
        let current_sha = repo.get_current_sha()?;

        assert!(!current_sha.is_empty());

        Ok(())
    }

    #[test]
    fn test_commit_changes() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;
        repo.create_branch("feat/my-new-feature")?;
        repo.checkout("feat/my-new-feature")?;

        // create a file and commit
        let file_path = workspace_path.join("README.md");
        std::fs::write(&file_path, "Hello, world!").expect("Failed to write Readme file");
        repo.add(file_path.display().to_string().as_str())?;
        let commit_id = repo.commit_changes("feat: add README.md")?;

        assert!(!commit_id.is_empty());

        Ok(())
    }

    #[test]
    fn test_add_all() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;
        repo.create_branch("feat/my-new-feature")?;
        repo.checkout("feat/my-new-feature")?;

        // create a file and commit
        let file_path = workspace_path.join("README.md");
        std::fs::write(&file_path, "Hello, world!").expect("Failed to write Readme file");
        let commit_id = repo.add_all()?.commit("feat: add README.md")?;

        assert!(!commit_id.is_empty());

        Ok(())
    }

    #[test]
    fn test_get_previous_sha_without_parent() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        let parent_sha = repo.get_previous_sha()?;

        assert!(!parent_sha.is_empty());

        Ok(())
    }

    #[test]
    fn test_get_previous_sha() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Create a file and commit it to ensure we have a parent commit
        let file_path = workspace_path.join("README.md");
        std::fs::write(&file_path, "Hello, world!").expect("Failed to write Readme file");
        repo.add_all()?;
        repo.commit("feat: add file to test parent")?;

        // Now we should have a parent commit
        let parent_sha = repo.get_previous_sha()?;

        assert!(!parent_sha.is_empty());

        Ok(())
    }

    #[test]
    fn test_status() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();
        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // create a file and commit
        let file_path = workspace_path.join("README.md");
        std::fs::write(&file_path, "Hello, world!").expect("Failed to write Readme file");

        let status = repo.status_porcelain()?;

        assert!(!status.is_empty());

        Ok(())
    }

    #[test]
    fn test_get_branch_from_commit() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();
        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // create a file and commit
        let file_path = workspace_path.join("README.md");
        std::fs::write(&file_path, "Hello, world!").expect("Failed to write Readme file");
        let commit_id = repo.add_all()?.commit("feat: add README.md")?;

        let branch = repo.get_branch_from_commit(commit_id.as_str())?;

        assert_eq!(branch, Some(String::from("main")));

        Ok(())
    }

    #[test]
    fn test_get_all_files_changed_since_sha_with_status() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();
        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Create and commit the first file
        let file_path = workspace_path.join("README.md");
        std::fs::write(&file_path, "Hello, world!").expect("Failed to write Readme file");
        let first_commit_id = repo.add_all()?.commit("feat: add README.md")?;

        // Save the first commit ID to compare against

        // Make additional changes after the commit:
        // 1. Create a new file
        let new_file_path = workspace_path.join("NEW_FILE.md");
        std::fs::write(&new_file_path, "This is a new file").expect("Failed to write new file");

        // 2. Modify the existing file
        std::fs::write(&file_path, "Hello, world updated!").expect("Failed to update Readme file");

        // 3. Create a file that will be deleted - ENSURE it gets committed
        let delete_file_path = workspace_path.join("TEMP_FILE.md");
        std::fs::write(&delete_file_path, "This will be deleted")
            .expect("Failed to write temp file");

        // Commit these changes - make sure all files are added
        repo.add_all()?.commit("feat: more changes")?;

        // Now delete the temp file
        std::fs::remove_file(&delete_file_path).expect("Failed to delete temp file");

        // Make sure Git knows about the deletion by adding the deletion to the index
        repo.add_all()?;

        // Commit the deletion
        repo.commit("feat: deleted temp file")?;

        // Now get files changed since the first commit
        let changed_files_with_status =
            repo.get_all_files_changed_since_sha_with_status(&first_commit_id)?;

        // Check that we have changes
        assert!(!changed_files_with_status.is_empty());

        // Verify we have the expected changes
        let mut has_added = false;
        let mut has_modified = false;
        let mut has_deleted = false;

        for change in &changed_files_with_status {
            match change.status {
                GitFileStatus::Added => {
                    has_added = true;
                    assert!(change.path.contains("NEW_FILE.md"));
                }
                GitFileStatus::Modified => {
                    has_modified = true;
                    assert!(change.path.contains("README.md"));
                }
                GitFileStatus::Deleted => {
                    has_deleted = true;
                    assert!(change.path.contains("TEMP_FILE.md"));
                }
            }
        }

        // Verify we found all types of changes
        assert!(has_added, "Should have found an added file");
        assert!(has_modified, "Should have found a modified file");
        assert!(has_deleted, "Should have found a deleted file");

        Ok(())
    }

    #[test]
    fn test_get_all_files_changed_since_sha() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();
        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Create and commit the first file
        let file_path = workspace_path.join("README.md");
        std::fs::write(&file_path, "Hello, world!").expect("Failed to write Readme file");
        let first_commit_id = repo.add_all()?.commit("feat: add README.md")?;

        // Save the first commit ID to compare against

        // Make additional changes after the commit:
        // 1. Create a new file
        let new_file_path = workspace_path.join("NEW_FILE.md");
        std::fs::write(&new_file_path, "This is a new file").expect("Failed to write new file");

        // 2. Modify the existing file
        std::fs::write(&file_path, "Hello, world updated!").expect("Failed to update Readme file");

        // 3. Create a file that will be deleted - ENSURE it gets committed
        let delete_file_path = workspace_path.join("TEMP_FILE.md");
        std::fs::write(&delete_file_path, "This will be deleted")
            .expect("Failed to write temp file");

        // Commit these changes - make sure all files are added
        repo.add_all()?.commit("feat: more changes")?;

        // Now delete the temp file
        std::fs::remove_file(&delete_file_path).expect("Failed to delete temp file");

        // Make sure Git knows about the deletion by adding the deletion to the index
        repo.add_all()?;

        // Commit the deletion
        repo.commit("feat: deleted temp file")?;

        // Now get files changed since the first commit
        let changed_files = repo.get_all_files_changed_since_sha(&first_commit_id)?;

        assert!(!changed_files.is_empty());

        Ok(())
    }

    #[test]
    fn test_get_commits_since() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();
        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Initial commit is already created by Repo::create

        // 1. Create a file and make the first commit
        let readme_path = workspace_path.join("README.md");
        std::fs::write(&readme_path, "# Test Repository").expect("Failed to write README file");
        repo.add_all()?;
        let first_commit_id = repo.commit("feat: add README")?;

        // 2. Create a second file and make another commit
        let doc_path = workspace_path.join("docs");
        std::fs::create_dir_all(&doc_path).expect("Failed to create docs directory");
        let doc_file_path = doc_path.join("documentation.md");
        std::fs::write(&doc_file_path, "# Documentation")
            .expect("Failed to write documentation file");
        repo.add_all()?;
        let second_commit_id = repo.commit("feat: add documentation")?;

        // 3. Modify the README file and make a third commit
        std::fs::write(&readme_path, "# Test Repository\n\nUpdated content")
            .expect("Failed to update README file");
        repo.add_all()?;
        let third_commit_id = repo.commit("chore: update README")?;

        // Test 1: Get all commits (should return 4 commits: initial + our 3)
        let all_commits = repo.get_commits_since(None, &None)?;
        assert_eq!(all_commits.len(), 4, "Should have 4 commits in total");

        // Test 2: Get commits since first commit (should return 2 commits: second and third)
        let commits_since_first = repo.get_commits_since(Some(first_commit_id.clone()), &None)?;
        assert_eq!(commits_since_first.len(), 2, "Should have 2 commits since first_commit_id");

        // Verify the commits are in the correct order (newest first)
        assert_eq!(
            commits_since_first[0].hash, third_commit_id,
            "First commit should be the third commit"
        );
        assert_eq!(
            commits_since_first[1].hash, second_commit_id,
            "Second commit should be the second commit"
        );

        // Test 3: Get commits related to README.md (should return 2 commits: first and third)
        let readme_commits = repo.get_commits_since(None, &Some("README.md".to_string()))?;
        assert_eq!(readme_commits.len(), 2, "Should have 2 commits touching README.md");

        // Check that we have the right commits for README
        let readme_commit_hashes: Vec<String> =
            readme_commits.iter().map(|c| c.hash.clone()).collect();
        assert!(
            readme_commit_hashes.contains(&first_commit_id),
            "README commits should include first commit"
        );
        assert!(
            readme_commit_hashes.contains(&third_commit_id),
            "README commits should include third commit"
        );

        // Test 4: Get commits since first commit related to docs (should return 1 commit: second)
        let docs_commits_since_first =
            repo.get_commits_since(Some(first_commit_id.clone()), &Some("docs".to_string()))?;
        assert_eq!(
            docs_commits_since_first.len(),
            1,
            "Should have 1 commit touching docs since first commit"
        );
        assert_eq!(
            docs_commits_since_first[0].hash, second_commit_id,
            "Should be the second commit"
        );

        // Test 5: Verify commit details
        let example_commit = &all_commits[0]; // Most recent commit
        assert_eq!(example_commit.hash, third_commit_id, "Hash should match");
        assert!(!example_commit.author_name.is_empty(), "Author name should not be empty");
        assert!(!example_commit.author_email.is_empty(), "Author email should not be empty");
        assert!(!example_commit.author_date.is_empty(), "Author date should not be empty");
        assert_eq!(example_commit.message.trim(), "chore: update README", "Message should match");

        Ok(())
    }

    #[test]
    fn test_get_local_tags() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();
        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Create some local tags
        repo.create_tag("v1.0.0", None)?;
        repo.create_tag("v1.1.0", Some("Version 1.1.0".to_string()))?;
        repo.create_tag("v2.0.0-beta", Some("Beta release".to_string()))?;

        // Test getting local tags
        let local_tags = repo.get_remote_or_local_tags(Some(true))?;

        // Check the number of tags
        assert_eq!(local_tags.len(), 3, "Should have 3 local tags");

        // Verify tag names exist
        let tag_names: Vec<String> = local_tags.iter().map(|t| t.tag.clone()).collect();
        assert!(tag_names.contains(&"v1.0.0".to_string()), "Should contain v1.0.0 tag");
        assert!(tag_names.contains(&"v1.1.0".to_string()), "Should contain v1.1.0 tag");
        assert!(tag_names.contains(&"v2.0.0-beta".to_string()), "Should contain v2.0.0-beta tag");

        // Verify tags have valid hashes
        for tag in &local_tags {
            assert!(!tag.hash.is_empty(), "Tag hash should not be empty");
            // Make sure the hash is a valid hex string of appropriate length
            assert_eq!(tag.hash.len(), 40, "Tag hash should be 40 characters long");
            assert!(
                tag.hash.chars().all(|c| c.is_ascii_hexdigit()),
                "Tag hash should be hexadecimal"
            );
        }

        Ok(())
    }

    #[test]
    fn test_get_all_files_changed_since_branch() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();
        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Setup: Create a initial package structure
        // packages/
        // ├── pkg1/
        // │   ├── src/
        // │   │   └── index.js
        // │   └── package.json
        // ├── pkg2/
        // │   ├── src/
        // │   │   └── index.js
        // │   └── package.json
        // └── shared/
        //     └── utils.js

        // Create directory structure
        let packages_dir = workspace_path.join("packages");
        std::fs::create_dir_all(&packages_dir).expect("Failed to create packages dir");

        let pkg1_dir = packages_dir.join("pkg1");
        let pkg1_src = pkg1_dir.join("src");
        std::fs::create_dir_all(&pkg1_src).expect("Failed to create pkg1/src dir");

        let pkg2_dir = packages_dir.join("pkg2");
        let pkg2_src = pkg2_dir.join("src");
        std::fs::create_dir_all(&pkg2_src).expect("Failed to create pkg2/src dir");

        let shared_dir = packages_dir.join("shared");
        std::fs::create_dir_all(&shared_dir).expect("Failed to create shared dir");

        // Create initial files
        std::fs::write(pkg1_dir.join("package.json"), r#"{"name":"pkg1"}"#)
            .expect("Failed to write pkg1/package.json");
        std::fs::write(pkg1_src.join("index.js"), "console.log('pkg1');")
            .expect("Failed to write pkg1/src/index.js");
        std::fs::write(pkg2_dir.join("package.json"), r#"{"name":"pkg2"}"#)
            .expect("Failed to write pkg2/package.json");
        std::fs::write(pkg2_src.join("index.js"), "console.log('pkg2');")
            .expect("Failed to write pkg2/src/index.js");
        std::fs::write(shared_dir.join("utils.js"), "// Shared utilities")
            .expect("Failed to write shared/utils.js");

        // Initial commit with all files
        repo.add_all()?;
        repo.commit("feat: initial commit")?;

        // Create a feature branch from this point
        repo.create_branch("feature-branch")?;

        // Make changes on main branch:
        // 1. Modify pkg1/src/index.js
        std::fs::write(pkg1_src.join("index.js"), "console.log('pkg1 updated');")
            .expect("Failed to update pkg1/src/index.js");

        // 2. Add new file to pkg2
        std::fs::write(pkg2_src.join("new-file.js"), "// New file in pkg2")
            .expect("Failed to write pkg2/src/new-file.js");

        // 3. Update shared utils
        std::fs::write(shared_dir.join("utils.js"), "// Updated shared utilities")
            .expect("Failed to update shared/utils.js");

        // 4. Add a file outside of packages
        std::fs::write(workspace_path.join("root-file.txt"), "Root level file")
            .expect("Failed to write root file");

        // Commit the changes
        repo.add_all()?;
        repo.commit("feat: update files")?;

        // TEST CASES

        // Test 1: Get changes for all packages
        let package_paths = vec![packages_dir.to_string_lossy().to_string()];

        let changes = repo.get_all_files_changed_since_branch(&package_paths, "feature-branch")?;

        // Should include all package changes (3 files) but not root-file.txt
        assert_eq!(changes.len(), 3, "Should find 3 changed files in packages");

        // Check for expected files
        let has_pkg1_index = changes.iter().any(|f| f.contains("pkg1/src/index.js"));
        let has_pkg2_new_file = changes.iter().any(|f| f.contains("pkg2/src/new-file.js"));
        let has_shared_utils = changes.iter().any(|f| f.contains("shared/utils.js"));
        let has_root_file = changes.iter().any(|f| f.contains("root-file.txt"));

        assert!(has_pkg1_index, "Should include modified pkg1/src/index.js");
        assert!(has_pkg2_new_file, "Should include new pkg2/src/new-file.js");
        assert!(has_shared_utils, "Should include modified shared/utils.js");
        assert!(!has_root_file, "Should NOT include root-file.txt");

        // Test 2: Get changes only for pkg1
        let pkg1_paths = vec![pkg1_dir.to_string_lossy().to_string()];

        let pkg1_changes =
            repo.get_all_files_changed_since_branch(&pkg1_paths, "feature-branch")?;

        assert_eq!(pkg1_changes.len(), 1, "Should find 1 changed file in pkg1");
        assert!(pkg1_changes.iter().any(|f| f.contains("pkg1/src/index.js")));

        // Test 3: Get changes for multiple specific packages
        let specific_paths =
            vec![pkg1_dir.to_string_lossy().to_string(), pkg2_dir.to_string_lossy().to_string()];

        let specific_changes =
            repo.get_all_files_changed_since_branch(&specific_paths, "feature-branch")?;

        assert_eq!(specific_changes.len(), 2, "Should find 2 changed files in pkg1 and pkg2");
        assert!(specific_changes.iter().any(|f| f.contains("pkg1/src/index.js")));
        assert!(specific_changes.iter().any(|f| f.contains("pkg2/src/new-file.js")));
        assert!(!specific_changes.iter().any(|f| f.contains("shared/utils.js")));

        // Test 4: Non-existent package path
        let non_existent = vec![workspace_path.join("non-existent").to_string_lossy().to_string()];

        let non_existent_changes =
            repo.get_all_files_changed_since_branch(&non_existent, "feature-branch")?;

        assert_eq!(non_existent_changes.len(), 0, "Should find 0 files for non-existent path");

        // Test 5: Empty packages array
        let empty_packages: Vec<String> = vec![];
        let empty_changes =
            repo.get_all_files_changed_since_branch(&empty_packages, "feature-branch")?;

        assert_eq!(empty_changes.len(), 0, "Should find 0 files for empty packages list");

        // Test 6: Invalid branch name
        let result = repo.get_all_files_changed_since_branch(&package_paths, "non-existent-branch");
        assert!(result.is_err(), "Should return error for non-existent branch");

        Ok(())
    }

    #[test]
    fn test_get_all_files_changed_since_branch_with_duplicates() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();
        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Create a simpler structure with overlapping paths
        std::fs::create_dir_all(workspace_path.join("frontend/components"))
            .expect("Failed to create directories");
        std::fs::create_dir_all(workspace_path.join("frontend/utils"))
            .expect("Failed to create directories");

        // Create initial files
        std::fs::write(workspace_path.join("frontend/components/Button.js"), "// Button component")
            .expect("Failed to write Button.js");

        std::fs::write(workspace_path.join("frontend/utils/format.js"), "// Format utilities")
            .expect("Failed to write format.js");

        // Initial commit
        repo.add_all()?;
        repo.commit("feat: initial frontend")?;

        // Create feature branch
        repo.create_branch("feature")?;

        // Make changes
        std::fs::write(
            workspace_path.join("frontend/components/Button.js"),
            "// Updated Button component",
        )
        .expect("Failed to update Button.js");

        // Add a new file
        std::fs::write(workspace_path.join("frontend/components/Card.js"), "// Card component")
            .expect("Failed to write Card.js");

        // Commit changes
        repo.add_all()?;
        repo.commit("feat: update frontend components")?;

        // Test with overlapping package paths
        let package_paths = vec![
            workspace_path.join("frontend").to_string_lossy().to_string(),
            workspace_path.join("frontend/components").to_string_lossy().to_string(),
        ];

        let changes = repo.get_all_files_changed_since_branch(&package_paths, "feature")?;

        // Despite having two package paths that both match the changed files,
        // we should only get each file once in the result
        assert_eq!(changes.len(), 2, "Should find 2 unique files despite overlapping paths");

        // Check the specific files
        let has_button = changes.iter().any(|f| f.contains("Button.js"));
        let has_card = changes.iter().any(|f| f.contains("Card.js"));

        assert!(has_button, "Should include Button.js");
        assert!(has_card, "Should include Card.js");

        Ok(())
    }

    #[test]
    fn test_get_diverged_commit() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();
        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Initial commit is already created by Repo::create
        repo.get_current_sha()?;

        // Make a new commit on main
        let file_path = workspace_path.join("file1.txt");
        std::fs::write(&file_path, "initial content").expect("Failed to write file");
        repo.add_all()?;
        let main_commit = repo.commit("feat: add file1")?;

        // Create a branch from this point
        repo.create_branch("feature-branch")?;
        repo.checkout("feature-branch")?;

        // Make a commit on the feature branch
        let file2_path = workspace_path.join("file2.txt");
        std::fs::write(&file2_path, "feature content").expect("Failed to write file2");
        repo.add_all()?;
        let feature_commit = repo.commit("feat: add file2")?;

        // Switch back to main
        repo.checkout("main")?;

        // Make another commit on main
        std::fs::write(&file_path, "updated content").expect("Failed to update file");
        repo.add_all()?;
        let main_commit2 = repo.commit("feat: update file1")?;

        // Test with branch name
        let diverged_commit_by_branch = repo.get_diverged_commit("feature-branch")?;
        assert_eq!(
            diverged_commit_by_branch, main_commit,
            "Divergence point should be the commit where the branch was created"
        );

        // Test with commit SHA
        let diverged_commit_by_sha = repo.get_diverged_commit(&feature_commit)?;
        assert_eq!(
            diverged_commit_by_sha, main_commit,
            "Divergence point should be the same when using commit SHA"
        );

        // Test with HEAD reference
        let diverged_commit_with_head = repo.get_diverged_commit("HEAD")?;
        assert_eq!(
            diverged_commit_with_head, main_commit2,
            "Divergence with HEAD should be HEAD itself"
        );

        // Test with invalid reference
        let result = repo.get_diverged_commit("non-existent-branch");
        assert!(result.is_err(), "Should return error for invalid reference");

        Ok(())
    }

    #[test]
    #[ignore] // Ignore by default because it requires a remote repository
    fn test_fetch() -> Result<(), RepoError> {
        // This test requires a Git repository with a remote
        let current_dir = std::env::current_dir().unwrap();
        let repo = Repo::open(current_dir.to_str().unwrap())?;
        repo.config("Sublime Git Bot", "git-boot@websublime.com")?;

        // Simple fetch from origin
        let result = repo.fetch("origin", None, false)?;
        assert!(result, "Fetch should succeed");

        // Fetch with specific refspec
        let result =
            repo.fetch("origin", Some(&["refs/heads/main:refs/remotes/origin/main"]), false)?;
        assert!(result, "Fetch with specific refspec should succeed");

        // Fetch with pruning
        let result = repo.fetch("origin", None, true)?;
        assert!(result, "Fetch with pruning should succeed");

        Ok(())
    }
}
