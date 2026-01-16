use std::env;
use std::fs;
use std::path::PathBuf;

/// Gets the release version to use, with priority:
/// 1. Environment variable LOGOS_STORAGE_VERSION
/// 2. Cargo.toml metadata [package.metadata.prebuilt] libstorage
/// 3. "latest" (default)
pub fn get_release_version() -> Result<String, Box<dyn std::error::Error>> {
    println!("  [VERSION] Starting get_release_version");

    // Check for environment variable override (highest priority)
    println!("  [VERSION] Checking environment variable LOGOS_STORAGE_VERSION...");
    if let Ok(version) = env::var("LOGOS_STORAGE_VERSION") {
        println!("  [VERSION] ✓ Found version from environment: {}", version);
        println!(
            "  [VERSION] ✓ Using pinned version from environment: {}",
            version
        );
        return Ok(version);
    }
    println!("  [VERSION]   Environment variable not set");

    // Check for Cargo.toml metadata
    println!("  [VERSION] Checking Cargo.toml metadata...");
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir).join("Cargo.toml");
        println!("  [VERSION]   Manifest path: {}", manifest_path.display());

        if let Ok(content) = fs::read_to_string(&manifest_path) {
            println!("  [VERSION]   ✓ Cargo.toml read successfully");
            if let Some(version) = parse_metadata_version(&content) {
                println!("  [VERSION]   ✓ Found version from metadata: {}", version);
                println!(
                    "  [VERSION] ✓ Using pinned version from Cargo.toml metadata: {}",
                    version
                );
                return Ok(version);
            } else {
                println!("  [VERSION]   No version found in metadata");
            }
        } else {
            println!("  [VERSION]   ✗ Failed to read Cargo.toml");
        }
    } else {
        println!("  [VERSION]   ✗ CARGO_MANIFEST_DIR not set");
    }

    // Default to latest release
    println!("  [VERSION] ✓ Using latest release");
    println!("  [VERSION] ✓ get_release_version completed successfully");
    Ok("latest".to_string())
}

/// Parses the libstorage version from Cargo.toml metadata
pub fn parse_metadata_version(cargo_toml: &str) -> Option<String> {
    cargo_toml
        .lines()
        .find(|line| line.contains("[package.metadata.prebuilt]"))
        .and_then(|_| {
            cargo_toml
                .lines()
                .skip_while(|line| !line.contains("[package.metadata.prebuilt]"))
                .skip(1)
                .take_while(|line| !line.starts_with('['))
                .find(|line| line.trim().starts_with("libstorage"))
                .and_then(|line| {
                    line.split('=')
                        .nth(1)
                        .map(|v| v.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata_version() {
        let cargo_toml = r#"
[package]
name = "test"

[package.metadata.prebuilt]
libstorage = "master-60861d6a"
"#;
        assert_eq!(
            parse_metadata_version(cargo_toml),
            Some("master-60861d6a".to_string())
        );
    }

    #[test]
    fn test_parse_metadata_version_missing() {
        let cargo_toml = r#"
[package]
name = "test"
"#;
        assert_eq!(parse_metadata_version(cargo_toml), None);
    }
}
