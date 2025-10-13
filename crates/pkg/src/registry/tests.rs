#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod registry_tests {
    use crate::registry::{AuthType, PublishOptions, RegistryAuth, RegistryClient};

    #[test]
    fn test_registry_client_creation() {
        let client = RegistryClient::new("https://registry.npmjs.org".to_string(), None, 30, 3);
        assert!(client.is_ok());
    }

    #[test]
    fn test_publish_options_default() {
        let options = PublishOptions::default();
        assert_eq!(options.access, "public");
        assert_eq!(options.tag, "latest");
        assert!(!options.dry_run);
    }

    #[test]
    fn test_registry_auth_default() {
        let auth = RegistryAuth::default();
        assert!(matches!(auth.auth_type, AuthType::None));
        assert!(auth.token.is_none());
        assert!(auth.password.is_none());
    }

    #[test]
    fn test_auth_headers_token() {
        let auth = RegistryAuth {
            auth_type: AuthType::Token,
            token: Some("test-token".to_string()),
            password: None,
        };

        let client =
            RegistryClient::new("https://registry.npmjs.org".to_string(), Some(auth), 30, 3)
                .unwrap();

        let headers = client.get_auth_headers();
        assert!(headers.contains_key("Authorization"));
        assert!(headers["Authorization"].contains("Bearer test-token"));
    }

    #[test]
    fn test_auth_headers_basic() {
        let auth = RegistryAuth {
            auth_type: AuthType::Basic,
            token: Some("username".to_string()),
            password: Some("password".to_string()),
        };

        let client =
            RegistryClient::new("https://registry.npmjs.org".to_string(), Some(auth), 30, 3)
                .unwrap();

        let headers = client.get_auth_headers();
        assert!(headers.contains_key("Authorization"));
        assert!(headers["Authorization"].starts_with("Basic "));
    }

    #[test]
    fn test_auth_headers_none() {
        let client =
            RegistryClient::new("https://registry.npmjs.org".to_string(), None, 30, 3).unwrap();

        let headers = client.get_auth_headers();
        assert!(!headers.contains_key("Authorization"));
    }

    #[test]
    fn test_registry_client_properties() {
        let auth = RegistryAuth {
            auth_type: AuthType::Token,
            token: Some("test-token".to_string()),
            password: None,
        };

        let client =
            RegistryClient::new("https://custom-registry.com".to_string(), Some(auth), 60, 5)
                .unwrap();

        assert_eq!(client.base_url, "https://custom-registry.com");
        assert!(client.auth.is_some());
    }

    #[test]
    fn test_publish_options_custom() {
        let options = PublishOptions {
            access: "restricted".to_string(),
            tag: "beta".to_string(),
            dry_run: true,
            registry: Some("https://private-registry.com".to_string()),
            package_path: std::path::PathBuf::from("./my-package"),
        };

        assert_eq!(options.access, "restricted");
        assert_eq!(options.tag, "beta");
        assert!(options.dry_run);
        assert_eq!(options.registry, Some("https://private-registry.com".to_string()));
    }

    #[test]
    fn test_auth_type_variants() {
        assert_eq!(format!("{:?}", AuthType::None), "None");
        assert_eq!(format!("{:?}", AuthType::Token), "Token");
        assert_eq!(format!("{:?}", AuthType::Basic), "Basic");
    }

    #[test]
    fn test_registry_client_invalid_url() {
        let result = RegistryClient::new("invalid-url".to_string(), None, 30, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_client_zero_timeout() {
        let result = RegistryClient::new("https://registry.npmjs.org".to_string(), None, 0, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_client_zero_retries() {
        let result = RegistryClient::new("https://registry.npmjs.org".to_string(), None, 30, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_auth_serialization() {
        let auth = RegistryAuth {
            auth_type: AuthType::Token,
            token: Some("test-token".to_string()),
            password: None,
        };

        // Test JSON serialization
        let json_result = serde_json::to_string(&auth);
        assert!(json_result.is_ok());

        // Test JSON deserialization
        let json_str = json_result.unwrap();
        let deserialized: Result<RegistryAuth, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_auth = deserialized.unwrap();
        assert!(matches!(deserialized_auth.auth_type, AuthType::Token));
        assert_eq!(deserialized_auth.token, Some("test-token".to_string()));
        assert!(deserialized_auth.password.is_none());
    }

    #[test]
    fn test_publish_options_serialization() {
        let options = PublishOptions {
            access: "public".to_string(),
            tag: "latest".to_string(),
            dry_run: false,
            registry: Some("https://registry.npmjs.org".to_string()),
            package_path: std::path::PathBuf::from("./my-package"),
        };

        // Test JSON serialization
        let json_result = serde_json::to_string(&options);
        assert!(json_result.is_ok());

        // Test JSON deserialization
        let json_str = json_result.unwrap();
        let deserialized: Result<PublishOptions, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_options = deserialized.unwrap();
        assert_eq!(deserialized_options.access, "public");
        assert_eq!(deserialized_options.tag, "latest");
        assert!(!deserialized_options.dry_run);
    }

    #[test]
    fn test_auth_validation() {
        // Token auth requires token
        let auth = RegistryAuth {
            auth_type: AuthType::Token,
            token: None,
            password: Some("password".to_string()),
        };

        let result =
            RegistryClient::new("https://registry.npmjs.org".to_string(), Some(auth), 30, 3);
        assert!(result.is_err());

        // Basic auth requires both token (username) and password
        let auth = RegistryAuth {
            auth_type: AuthType::Basic,
            token: Some("username".to_string()),
            password: None,
        };

        let result =
            RegistryClient::new("https://registry.npmjs.org".to_string(), Some(auth), 30, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_publish_options_with_all_fields() {
        let options = PublishOptions {
            access: "restricted".to_string(),
            tag: "next".to_string(),
            dry_run: true,
            registry: Some("https://private.npmjs.org".to_string()),
            package_path: std::path::PathBuf::from("./my-package"),
        };

        assert_eq!(options.access, "restricted");
        assert_eq!(options.tag, "next");
        assert!(options.dry_run);
        assert!(options.registry.is_some());
    }
}
