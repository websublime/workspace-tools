#[cfg(test)]
mod error_tests {
    use sublime_hooks_tools::{HookError, HookResult};
    use sublime_git_tools::RepoError;
    use sublime_monorepo_tools::ChangeError;
    use sublime_package_tools::VersionError;

    #[test]
    fn test_error_as_ref() {
        let git_error = HookError::Git(RepoError::NotAGitRepository);
        assert_eq!(git_error.as_ref(), "HookErrorGit");

        let change_error = HookError::Change(ChangeError::NoGitRepository);
        assert_eq!(change_error.as_ref(), "HookErrorChange");

        let version_error = HookError::Version(VersionError::InvalidVersion("1.0".to_string()));
        assert_eq!(version_error.as_ref(), "HookErrorVersion");

        let hook_error = HookError::Hook("test error".to_string());
        assert_eq!(hook_error.as_ref(), "HookErrorHook");
    }

    #[test]
    fn test_error_display() {
        let git_error = HookError::Git(RepoError::NotAGitRepository);
        assert!(git_error.to_string().contains("Git error"));

        let change_error = HookError::Change(ChangeError::NoGitRepository);
        assert!(change_error.to_string().contains("Change error"));

        let version_error = HookError::Version(VersionError::InvalidVersion("1.0".to_string()));
        assert!(version_error.to_string().contains("Version error"));

        let hook_error = HookError::Hook("test error".to_string());
        assert!(hook_error.to_string().contains("Hook error"));
    }

    #[test]
    fn test_error_from() {
        // Test From<RepoError>
        let git_error = RepoError::NotAGitRepository;
        let hook_error: HookError = git_error.into();
        assert!(matches!(hook_error, HookError::Git(_)));

        // Test From<ChangeError>
        let change_error = ChangeError::NoGitRepository;
        let hook_error: HookError = change_error.into();
        assert!(matches!(hook_error, HookError::Change(_)));

        // Test From<VersionError>
        let version_error = VersionError::InvalidVersion("1.0".to_string());
        let hook_error: HookError = version_error.into();
        assert!(matches!(hook_error, HookError::Version(_)));
    }
} 