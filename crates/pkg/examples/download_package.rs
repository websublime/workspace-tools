//! # Package Download Example
//!
//! This example demonstrates how to use the NpmRegistry to download and extract
//! npm packages to a local directory.
//!
//! ## What
//! Shows the usage of `download_package` and `download_and_extract_package` methods
//! to retrieve packages from the npm registry and extract them locally.
//!
//! ## How
//! Creates an NpmRegistry instance, downloads a package tarball, and extracts
//! it to a destination directory with proper error handling.
//!
//! ## Why
//! Useful for package managers, build tools, or any application that needs
//! to download and extract npm packages locally.

#![allow(clippy::print_stdout)]
#![allow(clippy::unnecessary_wraps)]

use std::path::Path;
use sublime_package_tools::{NpmRegistry, PackageRegistry, PackageRegistryError};

/// Downloads and extracts a package to demonstrate the functionality
///
/// # Arguments
///
/// * `package_name` - Name of the package to download
/// * `version` - Version of the package to download
/// * `destination` - Directory where the package should be extracted
///
/// # Returns
///
/// `Ok(())` on success, or a `PackageRegistryError` if any operation fails
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
///
/// // Download and extract lodash to ./packages/lodash
/// download_and_extract_example("lodash", "4.17.21", Path::new("./packages/lodash"))?;
/// # Ok::<(), sublime_package_tools::PackageRegistryError>(())
/// ```
fn download_and_extract_example(
    package_name: &str,
    version: &str,
    destination: &Path,
) -> Result<(), PackageRegistryError> {
    println!("Creating npm registry client...");
    let registry = NpmRegistry::default();

    println!("Downloading package {package_name}@{version} to {destination:?}...");

    // Download and extract the package
    registry.download_and_extract_package(package_name, version, destination)?;

    println!("Successfully downloaded and extracted {package_name}@{version}");

    // Verify the extraction by checking if package.json exists
    let package_json_path = destination.join("package").join("package.json");
    if package_json_path.exists() {
        println!("✅ Package extracted successfully - package.json found at {package_json_path:?}");
    } else {
        println!("⚠️  Warning: package.json not found at expected location");
    }

    Ok(())
}

/// Downloads a package tarball without extracting to demonstrate download functionality
///
/// # Arguments
///
/// * `package_name` - Name of the package to download
/// * `version` - Version of the package to download
///
/// # Returns
///
/// `Ok(bytes)` containing the tarball data, or a `PackageRegistryError` if download fails
///
/// # Examples
///
/// ```no_run
/// // Download lodash tarball as bytes
/// let tarball_bytes = download_only_example("lodash", "4.17.21")?;
/// println!("Downloaded {} bytes", tarball_bytes.len());
/// # Ok::<(), sublime_package_tools::PackageRegistryError>(())
/// ```
fn download_only_example(
    package_name: &str,
    version: &str,
) -> Result<Vec<u8>, PackageRegistryError> {
    println!("Creating npm registry client...");
    let registry = NpmRegistry::default();

    println!("Downloading package {package_name}@{version} as bytes...");

    // Download the package tarball
    let tarball_bytes = registry.download_package(package_name, version)?;

    println!(
        "Successfully downloaded {}@{} ({} bytes)",
        package_name,
        version,
        tarball_bytes.len()
    );

    Ok(tarball_bytes)
}

/// Demonstrates downloading packages with custom registry configuration
///
/// # Arguments
///
/// * `package_name` - Name of the package to download
/// * `version` - Version of the package to download
/// * `registry_url` - Custom registry URL to use
/// * `destination` - Directory where the package should be extracted
///
/// # Returns
///
/// `Ok(())` on success, or a `PackageRegistryError` if any operation fails
fn custom_registry_example(
    package_name: &str,
    version: &str,
    registry_url: &str,
    destination: &Path,
) -> Result<(), PackageRegistryError> {
    println!("Creating custom npm registry client for {registry_url}...");

    // Create registry with custom URL and configuration
    let mut registry = NpmRegistry::new(registry_url);
    registry
        .set_user_agent("package-downloader-example/1.0.0")
        .set_cache_ttl(std::time::Duration::from_secs(600)); // 10 minutes cache

    println!("Download URL: {}", registry.get_download_url(package_name, version));

    println!("Downloading package {package_name}@{version} from custom registry...");

    // Download and extract the package
    registry.download_and_extract_package(package_name, version, destination)?;

    println!("Successfully downloaded {package_name}@{version} from {registry_url}");

    Ok(())
}

/// Main function demonstrating various download scenarios
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 NPM Package Download Examples\n");

    // Example 1: Download and extract a popular package
    println!("📦 Example 1: Download and extract lodash");
    let lodash_dest = Path::new("./temp_packages/lodash");
    match download_and_extract_example("lodash", "4.17.21", lodash_dest) {
        Ok(()) => println!("✅ Successfully downloaded lodash\n"),
        Err(e) => println!("❌ Failed to download lodash: {e}\n"),
    }

    // Example 2: Download a scoped package
    println!("📦 Example 2: Download and extract @types/node");
    let types_dest = Path::new("./temp_packages/types-node");
    match download_and_extract_example("@types/node", "18.15.0", types_dest) {
        Ok(()) => println!("✅ Successfully downloaded @types/node\n"),
        Err(e) => println!("❌ Failed to download @types/node: {e}\n"),
    }

    // Example 3: Download only (without extraction)
    println!("📦 Example 3: Download package bytes only");
    match download_only_example("express", "4.18.2") {
        Ok(bytes) => println!("✅ Successfully downloaded express ({} bytes)\n", bytes.len()),
        Err(e) => println!("❌ Failed to download express: {e}\n"),
    }

    // Example 4: Custom registry (will fail with default npmjs, but shows usage)
    println!("📦 Example 4: Custom registry example");
    let custom_dest = Path::new("./temp_packages/custom");
    match custom_registry_example(
        "lodash",
        "4.17.21",
        "https://registry.npmjs.org", // Using standard registry for demo
        custom_dest,
    ) {
        Ok(()) => println!("✅ Successfully used custom registry configuration\n"),
        Err(e) => println!("❌ Custom registry example failed: {e}\n"),
    }

    // Example 5: Error handling demonstration
    println!("📦 Example 5: Error handling - non-existent package");
    let error_dest = Path::new("./temp_packages/nonexistent");
    match download_and_extract_example(
        "this-package-definitely-does-not-exist",
        "1.0.0",
        error_dest,
    ) {
        Ok(()) => println!("✅ Unexpectedly succeeded"),
        Err(e) => println!("✅ Correctly handled error: {e}\n"),
    }

    println!("🎉 All examples completed!");
    println!("\n💡 Tips:");
    println!("  - Check ./temp_packages/ directory for extracted files");
    println!("  - Packages are extracted with 'package/' subdirectory structure");
    println!("  - Use custom registries for private packages");
    println!("  - Configure authentication for private registries");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_generation() {
        let registry = NpmRegistry::default();

        // Test regular package URL
        let url = registry.get_download_url("lodash", "4.17.21");
        assert_eq!(url, "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz");

        // Test scoped package URL
        let scoped_url = registry.get_download_url("@types/node", "18.15.0");
        assert_eq!(scoped_url, "https://registry.npmjs.org/@types/node/-/node-18.15.0.tgz");
    }

    #[test]
    fn test_custom_registry_url_generation() {
        let registry = NpmRegistry::new("https://my-registry.example.com");

        let url = registry.get_download_url("test-package", "1.0.0");
        assert_eq!(url, "https://my-registry.example.com/test-package/-/test-package-1.0.0.tgz");
    }

    #[test]
    fn test_registry_configuration() {
        let mut registry = NpmRegistry::new("https://example.com");

        registry
            .set_user_agent("test-agent/1.0.0")
            .set_cache_ttl(std::time::Duration::from_secs(300))
            .set_auth("test-token", "bearer");

        // Test that configuration methods return self for chaining
        let url = registry.get_download_url("test", "1.0.0");
        assert!(url.contains("example.com"));
    }
}
