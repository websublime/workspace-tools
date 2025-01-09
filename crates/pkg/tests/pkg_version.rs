#[cfg(test)]
mod version_tests {
    use semver::Version;
    use ws_pkg::version::Version as WsVersion;

    #[test]
    fn test_bump_major() {
        let version = WsVersion::bump_major("1.0.0");
        assert_eq!(version, Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn test_bump_minor() {
        let version = WsVersion::bump_minor("1.0.0");
        assert_eq!(version, Version::parse("1.1.0").unwrap());
    }

    #[test]
    fn test_bump_patch() {
        let version = WsVersion::bump_patch("1.0.0");
        assert_eq!(version, Version::parse("1.0.1").unwrap());
    }

    #[test]
    fn test_bump_snapshot() {
        let version = WsVersion::bump_snapshot("1.0.0", "sha");
        assert_eq!(version, Version::parse("1.0.0-alpha.sha").unwrap());
    }
}
