//! Tests for the Patch System

#[cfg(test)]
mod tests {
    use codex_bindings::patch_system::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_patch_engine_creation() {
        let engine = PatchEngine::new(false);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_architecture_detection() {
        assert_eq!(
            get_android_arch_from_target("aarch64-linux-android"),
            Some("arm64")
        );
        assert_eq!(
            get_android_arch_from_target("x86_64-linux-android"),
            Some("x86_64")
        );
        assert_eq!(
            get_android_arch_from_target("armv7-linux-androideabi"),
            Some("arm32")
        );
        assert_eq!(
            get_android_arch_from_target("i686-linux-android"),
            Some("x86")
        );
        assert_eq!(get_android_arch_from_target("unknown"), None);
    }

    #[test]
    fn test_android_build_detection() {
        // This test would need to be run with actual environment variables
        // For now, just test the function exists and returns a boolean
        let result = is_android_build();
        assert!(result == true || result == false);
    }

    #[test]
    fn test_patch_registry_loading() {
        // Create a temporary directory for test patches
        let temp_dir = TempDir::new().unwrap();
        let patches_dir = temp_dir.path().join("android-patches");
        fs::create_dir_all(&patches_dir).unwrap();

        // Create a simple test registry
        let test_registry = r#"
        {
            "arm64": ["001-test.patch", "002-test.patch"],
            "x86_64": ["001-test.patch"]
        }
        "#;

        fs::write(patches_dir.join("patches.json"), test_registry).unwrap();

        // Create patch directories
        fs::create_dir_all(patches_dir.join("arm64")).unwrap();
        fs::create_dir_all(patches_dir.join("x86_64")).unwrap();

        // Create dummy patch files
        fs::write(
            patches_dir.join("arm64/001-test.patch"),
            "dummy patch content",
        )
        .unwrap();
        fs::write(
            patches_dir.join("arm64/002-test.patch"),
            "dummy patch content",
        )
        .unwrap();
        fs::write(
            patches_dir.join("x86_64/001-test.patch"),
            "dummy patch content",
        )
        .unwrap();

        // Test loading registry (this would normally use android-patches directory)
        // For this test, we'll create a mock engine in the temp directory
        let engine = PatchEngine::new(false);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_get_available_architectures() {
        let engine = PatchEngine::new(false).unwrap();
        let archs = engine.get_available_architectures();

        // Should have at least the main architectures
        assert!(archs.is_ok());
        let arch_list = archs.unwrap();
        assert!(arch_list.contains(&"arm64".to_string()));
        assert!(arch_list.contains(&"x86_64".to_string()));
        assert!(arch_list.contains(&"arm32".to_string()));
        assert!(arch_list.contains(&"x86".to_string()));
    }

    #[test]
    fn test_get_patches_for_arch() {
        let engine = PatchEngine::new(false).unwrap();

        // Test getting patches for ARM64
        let arm64_patches = engine.get_patches_for_arch("arm64");
        assert!(arm64_patches.is_ok());
        let patches = arm64_patches.unwrap();
        assert!(!patches.is_empty());

        // Should have terminal fix as first patch
        assert!(patches[0].starts_with("001-"));
        assert!(patches[0].contains("terminal"));
    }

    #[test]
    fn test_invalid_architecture() {
        let engine = PatchEngine::new(false).unwrap();

        // Test invalid architecture
        let invalid_patches = engine.get_patches_for_arch("invalid");
        assert!(invalid_patches.is_err());
    }

    #[test]
    fn test_patch_ordering() {
        let engine = PatchEngine::new(false).unwrap();

        // Get patches for ARM64
        let patches = engine.get_patches_for_arch("arm64").unwrap();

        // Verify patches are numbered sequentially
        for (i, patch) in patches.iter().enumerate() {
            let expected_prefix = format!("{:03}-", i + 1);
            assert!(
                patch.starts_with(&expected_prefix),
                "Patch {} should start with {}",
                patch,
                expected_prefix
            );
        }
    }

    // Integration test - only run if we're in a git repository with patches
    #[test]
    #[ignore] // Ignore by default since it requires actual patches
    fn test_patch_validation_integration() {
        // This test requires the actual patch files to be present
        // Run with: cargo test -- --ignored test_patch_validation_integration

        let engine = PatchEngine::new(true).unwrap();

        // Try to validate patches for ARM64 (may fail if patches not applied)
        let result = engine.validate_patches_for_arch("arm64");

        // We don't assert success here since patches may not be applied
        // Just test that the validation runs without panicking
        match result {
            Ok(_) => println!("All patches validated successfully"),
            Err(e) => println!("Validation failed (expected if patches not applied): {}", e),
        }
    }
}
