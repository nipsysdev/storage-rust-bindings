//! Manifest download operations
//!
//! This module contains manifest-related download operations.

use crate::callback::{c_callback, CallbackFuture};
use crate::download::types::Manifest;
use crate::error::{CodexError, Result};
use crate::ffi::{codex_download_manifest, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;

/// Download manifest information for a content
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `cid` - The content ID to get manifest for
///
/// # Returns
///
/// The manifest information for the content
pub async fn download_manifest(node: &CodexNode, cid: &str) -> Result<Manifest> {
    if cid.is_empty() {
        return Err(CodexError::invalid_parameter("cid", "CID cannot be empty"));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_cid = string_to_c_string(cid);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_download_manifest(
            node.ctx as *mut _,
            c_cid,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_cid);
    }

    if result != 0 {
        return Err(CodexError::download_error("Failed to download manifest"));
    }

    // Wait for the operation to complete
    let manifest_json = future.await?;

    // Parse the manifest JSON
    let manifest: Manifest = serde_json::from_str(&manifest_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse manifest: {}", e)))?;

    Ok(manifest)
}

/// Validate a manifest structure
///
/// # Arguments
///
/// * `manifest` - The manifest to validate
///
/// # Returns
///
/// Ok(()) if the manifest is valid, or an error
pub fn validate_manifest(manifest: &Manifest) -> Result<()> {
    if manifest.cid.is_empty() {
        return Err(CodexError::invalid_parameter(
            "cid",
            "Manifest CID cannot be empty",
        ));
    }

    if manifest.size == 0 {
        return Err(CodexError::invalid_parameter(
            "size",
            "Manifest size must be greater than 0",
        ));
    }

    if manifest.blocks == 0 {
        return Err(CodexError::invalid_parameter(
            "blocks",
            "Manifest blocks must be greater than 0",
        ));
    }

    if manifest.created.is_empty() {
        return Err(CodexError::invalid_parameter(
            "created",
            "Manifest created timestamp cannot be empty",
        ));
    }

    Ok(())
}

/// Check if a manifest is likely to be accessible based on its metadata
///
/// # Arguments
///
/// * `manifest` - The manifest to check
///
/// # Returns
///
/// true if the manifest appears accessible, false otherwise
pub fn is_manifest_accessible(manifest: &Manifest) -> bool {
    // Check if the manifest was accessed recently (within last 30 days)
    if let Some(ref accessed) = manifest.accessed {
        if let Ok(accessed_time) = chrono::DateTime::parse_from_rfc3339(accessed) {
            let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);
            if accessed_time.naive_utc() > thirty_days_ago.naive_utc() {
                return true;
            }
        }
    }

    // Check if the content type is supported
    if let Some(ref content_type) = manifest.content_type {
        let supported_types = vec![
            "application/octet-stream",
            "text/plain",
            "application/json",
            "image/jpeg",
            "image/png",
            "application/pdf",
        ];

        if supported_types.contains(&content_type.as_str()) {
            return true;
        }
    }

    // Default to true if we can't determine accessibility
    true
}

/// Get the estimated download time for a manifest based on its size
///
/// # Arguments
///
/// * `manifest` - The manifest to estimate for
/// * `speed_bps` - Estimated download speed in bytes per second
///
/// # Returns
///
/// Estimated download time in seconds
pub fn estimate_download_time(manifest: &Manifest, speed_bps: f64) -> f64 {
    if speed_bps <= 0.0 {
        return f64::INFINITY;
    }

    manifest.size as f64 / speed_bps
}

/// Get the optimal chunk size for downloading a manifest
///
/// # Arguments
///
/// * `manifest` - The manifest to get chunk size for
///
/// # Returns
///
/// Recommended chunk size in bytes
pub fn get_optimal_chunk_size(manifest: &Manifest) -> usize {
    // Base chunk size on total size and number of blocks
    let avg_block_size = manifest.size / manifest.blocks;

    // Use a chunk size that's roughly 4x the average block size
    // but clamp it between 64KB and 4MB
    let recommended = avg_block_size * 4;

    std::cmp::max(
        64 * 1024,                                   // Minimum 64KB
        std::cmp::min(recommended, 4 * 1024 * 1024), // Maximum 4MB
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::CodexConfig;
    use serde_json::json;

    #[tokio::test]
    async fn test_download_manifest() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let manifest = download_manifest(&node, "QmExample").await;
        assert!(manifest.is_ok());

        let manifest = manifest.unwrap();
        assert_eq!(manifest.cid, "QmExample");
        assert_eq!(manifest.size, 1024);
        assert_eq!(manifest.blocks, 4);
    }

    #[tokio::test]
    async fn test_download_manifest_empty_cid() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        let result = download_manifest(&node, "").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CodexError::InvalidParameter { parameter, .. } => {
                assert_eq!(parameter, "cid");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_validate_manifest() {
        let valid_manifest = Manifest {
            cid: "QmExample".to_string(),
            size: 1024,
            blocks: 4,
            created: "2023-01-01T00:00:00Z".to_string(),
            accessed: Some("2023-01-02T00:00:00Z".to_string()),
            content_type: Some("text/plain".to_string()),
            metadata: Some(json!({"key": "value"})),
        };

        assert!(validate_manifest(&valid_manifest).is_ok());

        // Test invalid CID
        let mut invalid_manifest = valid_manifest.clone();
        invalid_manifest.cid = "".to_string();
        assert!(validate_manifest(&invalid_manifest).is_err());

        // Test invalid size
        invalid_manifest = valid_manifest.clone();
        invalid_manifest.size = 0;
        assert!(validate_manifest(&invalid_manifest).is_err());

        // Test invalid blocks
        invalid_manifest = valid_manifest.clone();
        invalid_manifest.blocks = 0;
        assert!(validate_manifest(&invalid_manifest).is_err());

        // Test invalid created
        invalid_manifest = valid_manifest.clone();
        invalid_manifest.created = "".to_string();
        assert!(validate_manifest(&invalid_manifest).is_err());
    }

    #[test]
    fn test_is_manifest_accessible() {
        let manifest = Manifest {
            cid: "QmExample".to_string(),
            size: 1024,
            blocks: 4,
            created: "2023-01-01T00:00:00Z".to_string(),
            accessed: Some("2023-12-01T00:00:00Z".to_string()), // Recent access
            content_type: Some("text/plain".to_string()),
            metadata: None,
        };

        assert!(is_manifest_accessible(&manifest));

        // Test with old access time
        let mut old_manifest = manifest.clone();
        old_manifest.accessed = Some("2020-01-01T00:00:00Z".to_string());
        assert!(!is_manifest_accessible(&old_manifest));

        // Test with supported content type
        let mut no_access_manifest = manifest.clone();
        no_access_manifest.accessed = None;
        no_access_manifest.content_type = Some("application/json".to_string());
        assert!(is_manifest_accessible(&no_access_manifest));

        // Test with unsupported content type
        let mut unsupported_manifest = manifest.clone();
        unsupported_manifest.accessed = None;
        unsupported_manifest.content_type = Some("application/unknown".to_string());
        assert!(is_manifest_accessible(&unsupported_manifest)); // Defaults to true
    }

    #[test]
    fn test_estimate_download_time() {
        let manifest = Manifest {
            cid: "QmExample".to_string(),
            size: 1024 * 1024, // 1MB
            blocks: 4,
            created: "2023-01-01T00:00:00Z".to_string(),
            accessed: None,
            content_type: None,
            metadata: None,
        };

        // Test with 1 MB/s speed
        let time = estimate_download_time(&manifest, 1024.0 * 1024.0);
        assert_eq!(time, 1.0);

        // Test with 0 speed
        let time = estimate_download_time(&manifest, 0.0);
        assert!(time.is_infinite());

        // Test with negative speed
        let time = estimate_download_time(&manifest, -1024.0);
        assert!(time.is_infinite());
    }

    #[test]
    fn test_get_optimal_chunk_size() {
        let manifest = Manifest {
            cid: "QmExample".to_string(),
            size: 1024 * 1024, // 1MB
            blocks: 4,         // Average block size: 256KB
            created: "2023-01-01T00:00:00Z".to_string(),
            accessed: None,
            content_type: None,
            metadata: None,
        };

        let chunk_size = get_optimal_chunk_size(&manifest);
        // Average block size is 256KB, 4x that is 1MB
        assert_eq!(chunk_size, 1024 * 1024);

        // Test with very small blocks
        let mut small_manifest = manifest.clone();
        small_manifest.size = 1024;
        small_manifest.blocks = 1024; // Average block size: 1 byte
        let chunk_size = get_optimal_chunk_size(&small_manifest);
        assert_eq!(chunk_size, 64 * 1024); // Minimum

        // Test with very large blocks
        let mut large_manifest = manifest.clone();
        large_manifest.size = 100 * 1024 * 1024; // 100MB
        large_manifest.blocks = 1; // Average block size: 100MB
        let chunk_size = get_optimal_chunk_size(&large_manifest);
        assert_eq!(chunk_size, 4 * 1024 * 1024); // Maximum
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = Manifest {
            cid: "QmExample".to_string(),
            size: 1024,
            blocks: 4,
            created: "2023-01-01T00:00:00Z".to_string(),
            accessed: Some("2023-01-02T00:00:00Z".to_string()),
            content_type: Some("text/plain".to_string()),
            metadata: Some(json!({"key": "value"})),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.cid, deserialized.cid);
        assert_eq!(manifest.size, deserialized.size);
        assert_eq!(manifest.blocks, deserialized.blocks);
        assert_eq!(manifest.created, deserialized.created);
        assert_eq!(manifest.accessed, deserialized.accessed);
        assert_eq!(manifest.content_type, deserialized.content_type);
        assert_eq!(manifest.metadata, deserialized.metadata);
    }
}
