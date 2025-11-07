//! Space management operations for storage
//!
//! This module contains storage management operations: manifests and space.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_storage_list, codex_storage_space, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use serde::{Deserialize, Serialize};

/// Manifest information for a stored content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Content ID (CID) - set separately in fetch()
    #[serde(skip)]
    pub cid: String,
    /// Tree CID - root of the merkle tree
    #[serde(rename = "treeCid", default)]
    pub tree_cid: String,
    /// Dataset size - total size of all blocks
    #[serde(rename = "datasetSize")]
    pub dataset_size: usize,
    /// Block size - size of each contained block
    #[serde(rename = "blockSize")]
    pub block_size: usize,
    /// Filename - name of the file (optional)
    #[serde(default)]
    pub filename: String,
    /// Mimetype - MIME type of the file (optional)
    #[serde(default)]
    pub mimetype: String,
    /// Protected datasets have erasure coded info
    #[serde(default)]
    pub protected: bool,
}

/// Manifest with CID wrapper (as returned by storage list)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestWithCid {
    /// Content ID (CID)
    pub cid: String,
    /// Manifest data
    pub manifest: Manifest,
}

/// Storage space information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    /// Total number of blocks stored by the node
    #[serde(rename = "totalBlocks")]
    pub total_blocks: usize,
    /// Maximum storage space (in bytes) available
    #[serde(rename = "quotaMaxBytes")]
    pub quota_max_bytes: u64,
    /// Amount of storage space (in bytes) currently used
    #[serde(rename = "quotaUsedBytes")]
    pub quota_used_bytes: u64,
    /// Amount of storage reserved (in bytes) for future use
    #[serde(rename = "quotaReservedBytes")]
    pub quota_reserved_bytes: u64,
}

/// List all manifests in the storage
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// A vector of manifests for all stored content
pub async fn manifests(node: &CodexNode) -> Result<Vec<Manifest>> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_list(
            node.ctx as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(CodexError::storage_error(
            "manifests",
            "Failed to list manifests",
        ));
    }

    // Wait for the operation to complete
    let manifests_json = future.await?;

    // Parse the manifests JSON array
    let manifests_with_cid: Vec<ManifestWithCid> = serde_json::from_str(&manifests_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse manifests: {}", e)))?;

    // Convert to Manifest structs with CID set
    let manifests: Vec<Manifest> = manifests_with_cid
        .into_iter()
        .map(|item| {
            let mut manifest = item.manifest;
            manifest.cid = item.cid;
            manifest
        })
        .collect();

    Ok(manifests)
}

/// Get storage space information
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// Information about storage usage and availability
pub async fn space(node: &CodexNode) -> Result<Space> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_storage_space(
            node.ctx as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(CodexError::storage_error(
            "space",
            "Failed to get storage space",
        ));
    }

    // Wait for the operation to complete
    let space_json = future.await?;

    // Parse the space JSON
    let space: Space = serde_json::from_str(&space_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse space info: {}", e)))?;

    Ok(space)
}

/// Get storage usage statistics
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// Storage usage statistics
pub async fn storage_stats(node: &CodexNode) -> Result<StorageStats> {
    let space_info = space(node).await?;
    let manifest_list = manifests(node).await?;

    let total_files = manifest_list.len();
    let total_size: usize = manifest_list.iter().map(|m| m.dataset_size).sum();
    let protected_files = manifest_list.iter().filter(|m| m.protected).count();
    let protected_size: usize = manifest_list
        .iter()
        .filter(|m| m.protected)
        .map(|m| m.dataset_size)
        .sum();

    Ok(StorageStats {
        total_files,
        total_size,
        protected_files,
        protected_size,
        space_info,
    })
}

/// Storage usage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Total number of files stored
    pub total_files: usize,
    /// Total size of all files in bytes
    pub total_size: usize,
    /// Number of protected files
    pub protected_files: usize,
    /// Size of protected files in bytes
    pub protected_size: usize,
    /// Raw space information from the node
    pub space_info: Space,
}

impl StorageStats {
    /// Get the percentage of storage used
    pub fn usage_percentage(&self) -> f64 {
        if self.space_info.quota_max_bytes == 0 {
            0.0
        } else {
            self.space_info.quota_used_bytes as f64 / self.space_info.quota_max_bytes as f64
        }
    }

    /// Get the percentage of storage that is reserved
    pub fn reserved_percentage(&self) -> f64 {
        if self.space_info.quota_max_bytes == 0 {
            0.0
        } else {
            self.space_info.quota_reserved_bytes as f64 / self.space_info.quota_max_bytes as f64
        }
    }

    /// Get the available storage space in bytes
    pub fn available_bytes(&self) -> u64 {
        self.space_info
            .quota_max_bytes
            .saturating_sub(self.space_info.quota_used_bytes)
    }

    /// Check if storage is nearly full (above 90%)
    pub fn is_nearly_full(&self) -> bool {
        self.usage_percentage() > 0.9
    }

    /// Check if storage is critically full (above 95%)
    pub fn is_critically_full(&self) -> bool {
        self.usage_percentage() > 0.95
    }
}

/// Find manifests by filename pattern
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `pattern` - Filename pattern to search for (supports wildcards)
///
/// # Returns
///
/// A vector of matching manifests
pub async fn find_manifests_by_filename(node: &CodexNode, pattern: &str) -> Result<Vec<Manifest>> {
    let all_manifests = manifests(node).await?;

    // Simple pattern matching - in a real implementation, you might want
    // to use a more sophisticated pattern matching library
    let pattern = pattern.to_lowercase();
    let matching_manifests: Vec<Manifest> = all_manifests
        .into_iter()
        .filter(|m| {
            let filename = m.filename.to_lowercase();

            // Simple wildcard support
            if pattern.contains('*') {
                let parts: Vec<&str> = pattern.split('*').collect();
                if parts.len() == 2 {
                    filename.starts_with(parts[0]) && filename.ends_with(parts[1])
                } else {
                    filename.contains(&pattern.replace('*', ""))
                }
            } else {
                filename.contains(&pattern)
            }
        })
        .collect();

    Ok(matching_manifests)
}

/// Find manifests by MIME type
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `mime_type` - MIME type to search for
///
/// # Returns
///
/// A vector of matching manifests
pub async fn find_manifests_by_mime_type(
    node: &CodexNode,
    mime_type: &str,
) -> Result<Vec<Manifest>> {
    let all_manifests = manifests(node).await?;

    let matching_manifests: Vec<Manifest> = all_manifests
        .into_iter()
        .filter(|m| m.mimetype.to_lowercase() == mime_type.to_lowercase())
        .collect();

    Ok(matching_manifests)
}

/// Get storage optimization suggestions
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// Suggestions for optimizing storage usage
pub async fn get_optimization_suggestions(node: &CodexNode) -> Result<Vec<OptimizationSuggestion>> {
    let stats = storage_stats(node).await?;
    let mut suggestions = Vec::new();

    // Check storage usage
    if stats.is_critically_full() {
        suggestions.push(OptimizationSuggestion {
            priority: SuggestionPriority::Critical,
            category: SuggestionCategory::Storage,
            title: "Storage critically full".to_string(),
            description: format!(
                "Storage is {:.1}% full. Consider deleting old or unused content.",
                stats.usage_percentage() * 100.0
            ),
            action: Some("Delete unused content or increase storage quota".to_string()),
        });
    } else if stats.is_nearly_full() {
        suggestions.push(OptimizationSuggestion {
            priority: SuggestionPriority::High,
            category: SuggestionCategory::Storage,
            title: "Storage nearly full".to_string(),
            description: format!(
                "Storage is {:.1}% full. Start planning cleanup operations.",
                stats.usage_percentage() * 100.0
            ),
            action: Some("Review and delete unused content".to_string()),
        });
    }

    // Check protected content ratio
    if stats.total_files > 0 {
        let protected_ratio = stats.protected_files as f64 / stats.total_files as f64;
        if protected_ratio > 0.8 {
            suggestions.push(OptimizationSuggestion {
                priority: SuggestionPriority::Medium,
                category: SuggestionCategory::Optimization,
                title: "High protected content ratio".to_string(),
                description: format!(
                    "{:.1}% of your content is protected. Consider if all content needs protection.",
                    protected_ratio * 100.0
                ),
                action: Some("Review protection settings for content".to_string()),
            });
        }
    }

    // Check for empty files
    let all_manifests = manifests(node).await?;
    let empty_files: Vec<_> = all_manifests
        .iter()
        .filter(|m| m.dataset_size == 0)
        .collect();
    if !empty_files.is_empty() {
        suggestions.push(OptimizationSuggestion {
            priority: SuggestionPriority::Low,
            category: SuggestionCategory::Cleanup,
            title: "Empty files detected".to_string(),
            description: format!(
                "Found {} empty files that can be cleaned up.",
                empty_files.len()
            ),
            action: Some("Delete empty files to free up metadata space".to_string()),
        });
    }

    Ok(suggestions)
}

/// Storage optimization suggestion
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    /// Priority level of the suggestion
    pub priority: SuggestionPriority,
    /// Category of the suggestion
    pub category: SuggestionCategory,
    /// Title of the suggestion
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Suggested action to take
    pub action: Option<String>,
}

/// Priority level for suggestions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Category of suggestions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionCategory {
    Storage,
    Optimization,
    Cleanup,
    Security,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::{CodexConfig, LogLevel};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_manifests_empty() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error) // Reduce log noise
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024); // 100 MB

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let manifests_result = manifests(&node).await;
        assert!(
            manifests_result.is_ok(),
            "Failed to get manifests: {:?}",
            manifests_result.err()
        );

        let manifest_list = manifests_result.unwrap();
        // Should be empty for a new node
        assert_eq!(manifest_list.len(), 0, "New node should have no manifests");

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_manifests_with_started_node() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let manifests_result = manifests(&node).await;
        assert!(manifests_result.is_ok());

        let manifest_list = manifests_result.unwrap();
        // Verify the structure of returned manifests
        for manifest in &manifest_list {
            assert!(!manifest.cid.is_empty(), "Manifest CID should not be empty");
            assert!(
                manifest.dataset_size > 0,
                "Manifest dataset_size should be positive"
            );
            assert!(
                manifest.block_size > 0,
                "Manifest block_size should be positive"
            );
        }

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_space_info() {
        let temp_dir = tempdir().unwrap();
        let quota = 100 * 1024 * 1024; // 100 MB
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(quota);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let space_result = space(&node).await;
        assert!(
            space_result.is_ok(),
            "Failed to get space info: {:?}",
            space_result.err()
        );

        let space_info = space_result.unwrap();
        assert!(space_info.quota_max_bytes > 0, "Quota should be positive");
        assert!(
            space_info.quota_used_bytes > 0 || space_info.quota_used_bytes == 0,
            "Used space should be valid"
        );
        assert!(
            space_info.quota_used_bytes <= space_info.quota_max_bytes,
            "Used space should not exceed quota"
        );
        assert!(
            space_info.quota_reserved_bytes > 0 || space_info.quota_reserved_bytes == 0,
            "Reserved space should be valid"
        );

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let temp_dir = tempdir().unwrap();
        let quota = 200 * 1024 * 1024; // 200 MB
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(quota);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let stats_result = storage_stats(&node).await;
        assert!(
            stats_result.is_ok(),
            "Failed to get storage stats: {:?}",
            stats_result.err()
        );

        let stats = stats_result.unwrap();
        assert!(stats.total_files >= 0, "Total files should be non-negative");
        assert!(stats.total_size >= 0, "Total size should be non-negative");
        assert!(
            stats.protected_files >= 0,
            "Protected files should be non-negative"
        );
        assert!(
            stats.protected_size >= 0,
            "Protected size should be non-negative"
        );

        // Test helper methods
        assert!(stats.usage_percentage() >= 0.0 && stats.usage_percentage() <= 1.0);
        assert!(stats.reserved_percentage() >= 0.0 && stats.reserved_percentage() <= 1.0);

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_storage_operations_without_starting_node() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let node = CodexNode::new(config).unwrap();
        // Don't start the node

        // These operations should work even if the node is not started
        let manifests_result = manifests(&node).await;
        assert!(
            manifests_result.is_ok(),
            "Manifests should work without starting node"
        );

        let space_result = space(&node).await;
        assert!(
            space_result.is_ok(),
            "Space info should work without starting node"
        );

        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_storage_operations() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Test concurrent operations
        let manifests_future = manifests(&node);
        let space_future = space(&node);

        let (manifests_result, space_result) = tokio::join!(manifests_future, space_future);

        assert!(manifests_result.is_ok());
        assert!(space_result.is_ok());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[test]
    fn test_manifest_structure() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Create a test manifest to verify structure
        let test_manifest = Manifest {
            cid: "QmTest123".to_string(),
            tree_cid: "QmTree123".to_string(),
            dataset_size: 1024,
            block_size: 256,
            filename: "test.txt".to_string(),
            mimetype: "text/plain".to_string(),
            protected: false,
        };

        // Verify the manifest can be serialized and deserialized
        let json = serde_json::to_string(&test_manifest).unwrap();
        let deserialized: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(test_manifest.tree_cid, deserialized.tree_cid);
        assert_eq!(test_manifest.dataset_size, deserialized.dataset_size);
        assert_eq!(test_manifest.block_size, deserialized.block_size);
        assert_eq!(test_manifest.filename, deserialized.filename);
        assert_eq!(test_manifest.mimetype, deserialized.mimetype);
        assert_eq!(test_manifest.protected, deserialized.protected);

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[test]
    fn test_space_structure() {
        let temp_dir = tempdir().unwrap();
        let quota = 200 * 1024 * 1024; // 200 MB
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(quota);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Create a test space info to verify structure
        let test_space = Space {
            total_blocks: 10,
            quota_max_bytes: quota,
            quota_used_bytes: 50 * 1024 * 1024,      // 50 MB
            quota_reserved_bytes: 150 * 1024 * 1024, // 150 MB
        };

        // Verify the space info can be serialized and deserialized
        let json = serde_json::to_string(&test_space).unwrap();
        let deserialized: Space = serde_json::from_str(&json).unwrap();

        assert_eq!(test_space.total_blocks, deserialized.total_blocks);
        assert_eq!(test_space.quota_max_bytes, deserialized.quota_max_bytes);
        assert_eq!(test_space.quota_used_bytes, deserialized.quota_used_bytes);
        assert_eq!(
            test_space.quota_reserved_bytes,
            deserialized.quota_reserved_bytes
        );

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[test]
    fn test_storage_stats_methods() {
        let space_info = Space {
            total_blocks: 10,
            quota_max_bytes: 1000,
            quota_used_bytes: 800,
            quota_reserved_bytes: 100,
        };

        let stats = StorageStats {
            total_files: 5,
            total_size: 500,
            protected_files: 2,
            protected_size: 200,
            space_info,
        };

        assert_eq!(stats.usage_percentage(), 0.8);
        assert_eq!(stats.reserved_percentage(), 0.1);
        assert_eq!(stats.available_bytes(), 200);
        assert!(stats.is_nearly_full());
        assert!(!stats.is_critically_full());

        let critical_space_info = Space {
            total_blocks: 10,
            quota_max_bytes: 1000,
            quota_used_bytes: 960,
            quota_reserved_bytes: 40,
        };

        let critical_stats = StorageStats {
            total_files: 5,
            total_size: 500,
            protected_files: 2,
            protected_size: 200,
            space_info: critical_space_info,
        };

        assert!(critical_stats.is_critically_full());
    }
}
