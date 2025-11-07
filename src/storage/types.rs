//! Types for storage operations

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

impl Manifest {
    /// Create a new manifest
    pub fn new(cid: String) -> Self {
        Self {
            cid,
            tree_cid: String::new(),
            dataset_size: 0,
            block_size: 0,
            filename: String::new(),
            mimetype: String::new(),
            protected: false,
        }
    }

    /// Set the tree CID
    pub fn tree_cid(mut self, tree_cid: String) -> Self {
        self.tree_cid = tree_cid;
        self
    }

    /// Set the dataset size
    pub fn dataset_size(mut self, dataset_size: usize) -> Self {
        self.dataset_size = dataset_size;
        self
    }

    /// Set the block size
    pub fn block_size(mut self, block_size: usize) -> Self {
        self.block_size = block_size;
        self
    }

    /// Set the filename
    pub fn filename(mut self, filename: String) -> Self {
        self.filename = filename;
        self
    }

    /// Set the mimetype
    pub fn mimetype(mut self, mimetype: String) -> Self {
        self.mimetype = mimetype;
        self
    }

    /// Set whether the manifest is protected
    pub fn protected(mut self, protected: bool) -> Self {
        self.protected = protected;
        self
    }

    /// Get the estimated number of blocks based on dataset and block size
    pub fn estimated_blocks(&self) -> usize {
        if self.block_size == 0 {
            0
        } else {
            (self.dataset_size + self.block_size - 1) / self.block_size
        }
    }

    /// Check if the manifest is likely to be a file (has filename)
    pub fn is_file(&self) -> bool {
        !self.filename.is_empty()
    }

    /// Check if the manifest is likely to be directory data
    pub fn is_directory(&self) -> bool {
        self.filename.is_empty() && self.dataset_size > 0
    }

    /// Get the file extension if this is a file
    pub fn file_extension(&self) -> Option<String> {
        if self.is_file() {
            if let Some(dot_pos) = self.filename.rfind('.') {
                Some(self.filename[dot_pos + 1..].to_lowercase())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get a human-readable size string
    pub fn size_string(&self) -> String {
        bytesize::ByteSize::b(self.dataset_size as u64).to_string()
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            cid: String::new(),
            tree_cid: String::new(),
            dataset_size: 0,
            block_size: 0,
            filename: String::new(),
            mimetype: String::new(),
            protected: false,
        }
    }
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

impl Space {
    /// Create a new space info
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the total blocks
    pub fn total_blocks(mut self, total_blocks: usize) -> Self {
        self.total_blocks = total_blocks;
        self
    }

    /// Set the quota max bytes
    pub fn quota_max_bytes(mut self, quota_max_bytes: u64) -> Self {
        self.quota_max_bytes = quota_max_bytes;
        self
    }

    /// Set the quota used bytes
    pub fn quota_used_bytes(mut self, quota_used_bytes: u64) -> Self {
        self.quota_used_bytes = quota_used_bytes;
        self
    }

    /// Set the quota reserved bytes
    pub fn quota_reserved_bytes(mut self, quota_reserved_bytes: u64) -> Self {
        self.quota_reserved_bytes = quota_reserved_bytes;
        self
    }

    /// Get the available storage space in bytes
    pub fn available_bytes(&self) -> u64 {
        self.quota_max_bytes.saturating_sub(self.quota_used_bytes)
    }

    /// Get the usage percentage (0.0 to 1.0)
    pub fn usage_percentage(&self) -> f64 {
        if self.quota_max_bytes == 0 {
            0.0
        } else {
            self.quota_used_bytes as f64 / self.quota_max_bytes as f64
        }
    }

    /// Get the reserved percentage (0.0 to 1.0)
    pub fn reserved_percentage(&self) -> f64 {
        if self.quota_max_bytes == 0 {
            0.0
        } else {
            self.quota_reserved_bytes as f64 / self.quota_max_bytes as f64
        }
    }

    /// Check if storage is nearly full (above 90%)
    pub fn is_nearly_full(&self) -> bool {
        self.usage_percentage() > 0.9
    }

    /// Check if storage is critically full (above 95%)
    pub fn is_critically_full(&self) -> bool {
        self.usage_percentage() > 0.95
    }

    /// Get a human-readable string for quota max
    pub fn quota_max_string(&self) -> String {
        bytesize::ByteSize::b(self.quota_max_bytes).to_string()
    }

    /// Get a human-readable string for quota used
    pub fn quota_used_string(&self) -> String {
        bytesize::ByteSize::b(self.quota_used_bytes).to_string()
    }

    /// Get a human-readable string for available space
    pub fn available_string(&self) -> String {
        bytesize::ByteSize::b(self.available_bytes()).to_string()
    }
}

impl Default for Space {
    fn default() -> Self {
        Self {
            total_blocks: 0,
            quota_max_bytes: 0,
            quota_used_bytes: 0,
            quota_reserved_bytes: 0,
        }
    }
}

/// Storage operation result
#[derive(Debug, Clone)]
pub enum StorageResult<T> {
    /// Operation succeeded
    Success(T),
    /// Operation failed with an error
    Error(String),
    /// Operation was not found
    NotFound,
    /// Operation already exists
    AlreadyExists,
}

impl<T> StorageResult<T> {
    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, StorageResult::Success(_))
    }

    /// Check if the result is an error
    pub fn is_error(&self) -> bool {
        matches!(self, StorageResult::Error(_))
    }

    /// Check if the result is not found
    pub fn is_not_found(&self) -> bool {
        matches!(self, StorageResult::NotFound)
    }

    /// Check if the result already exists
    pub fn is_already_exists(&self) -> bool {
        matches!(self, StorageResult::AlreadyExists)
    }

    /// Get the success value if successful
    pub fn success_value(&self) -> Option<&T> {
        match self {
            StorageResult::Success(value) => Some(value),
            _ => None,
        }
    }

    /// Get the error message if error
    pub fn error_message(&self) -> Option<&String> {
        match self {
            StorageResult::Error(message) => Some(message),
            _ => None,
        }
    }

    /// Convert to Result type
    pub fn to_result(self) -> Result<T, String> {
        match self {
            StorageResult::Success(value) => Ok(value),
            StorageResult::Error(message) => Err(message),
            StorageResult::NotFound => Err("Not found".to_string()),
            StorageResult::AlreadyExists => Err("Already exists".to_string()),
        }
    }
}

/// Storage filter options
#[derive(Debug, Clone, Default)]
pub struct StorageFilter {
    /// Filter by filename pattern
    pub filename_pattern: Option<String>,
    /// Filter by MIME type
    pub mime_type: Option<String>,
    /// Filter by minimum size
    pub min_size: Option<usize>,
    /// Filter by maximum size
    pub max_size: Option<usize>,
    /// Filter by protection status
    pub protected: Option<bool>,
    /// Limit the number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

impl StorageFilter {
    /// Create a new storage filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the filename pattern
    pub fn filename_pattern(mut self, pattern: String) -> Self {
        self.filename_pattern = Some(pattern);
        self
    }

    /// Set the MIME type
    pub fn mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Set the minimum size
    pub fn min_size(mut self, size: usize) -> Self {
        self.min_size = Some(size);
        self
    }

    /// Set the maximum size
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = Some(size);
        self
    }

    /// Set the protection status
    pub fn protected(mut self, protected: bool) -> Self {
        self.protected = Some(protected);
        self
    }

    /// Set the limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Check if a manifest matches this filter
    pub fn matches(&self, manifest: &Manifest) -> bool {
        // Check filename pattern
        if let Some(ref pattern) = self.filename_pattern {
            let pattern = pattern.to_lowercase();
            let filename = manifest.filename.to_lowercase();

            if !filename.contains(&pattern) {
                return false;
            }
        }

        // Check MIME type
        if let Some(ref mime_type) = self.mime_type {
            if manifest.mimetype.to_lowercase() != mime_type.to_lowercase() {
                return false;
            }
        }

        // Check minimum size
        if let Some(min_size) = self.min_size {
            if manifest.dataset_size < min_size {
                return false;
            }
        }

        // Check maximum size
        if let Some(max_size) = self.max_size {
            if manifest.dataset_size > max_size {
                return false;
            }
        }

        // Check protection status
        if let Some(protected) = self.protected {
            if manifest.protected != protected {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = Manifest::new("QmTest".to_string())
            .tree_cid("QmTree".to_string())
            .dataset_size(1024)
            .block_size(256)
            .filename("test.txt".to_string())
            .mimetype("text/plain".to_string())
            .protected(true);

        assert_eq!(manifest.cid, "QmTest");
        assert_eq!(manifest.tree_cid, "QmTree");
        assert_eq!(manifest.dataset_size, 1024);
        assert_eq!(manifest.block_size, 256);
        assert_eq!(manifest.filename, "test.txt");
        assert_eq!(manifest.mimetype, "text/plain");
        assert!(manifest.protected);
    }

    #[test]
    fn test_manifest_methods() {
        let manifest = Manifest::new("QmTest".to_string())
            .dataset_size(1024)
            .block_size(256)
            .filename("test.txt".to_string());

        assert_eq!(manifest.estimated_blocks(), 4);
        assert!(manifest.is_file());
        assert!(!manifest.is_directory());
        assert_eq!(manifest.file_extension(), Some("txt".to_string()));
        assert!(manifest.size_string().contains("1.0"));
    }

    #[test]
    fn test_space_creation() {
        let space = Space::new()
            .total_blocks(10)
            .quota_max_bytes(1000)
            .quota_used_bytes(800)
            .quota_reserved_bytes(100);

        assert_eq!(space.total_blocks, 10);
        assert_eq!(space.quota_max_bytes, 1000);
        assert_eq!(space.quota_used_bytes, 800);
        assert_eq!(space.quota_reserved_bytes, 100);
        assert_eq!(space.available_bytes(), 200);
        assert_eq!(space.usage_percentage(), 0.8);
        assert_eq!(space.reserved_percentage(), 0.1);
        assert!(space.is_nearly_full());
        assert!(!space.is_critically_full());
    }

    #[test]
    fn test_storage_result() {
        let success: StorageResult<i32> = StorageResult::Success(42);
        assert!(success.is_success());
        assert!(!success.is_error());
        assert_eq!(success.success_value(), Some(&42));

        let error: StorageResult<i32> = StorageResult::Error("Failed".to_string());
        assert!(!error.is_success());
        assert!(error.is_error());
        assert_eq!(error.error_message(), Some(&"Failed".to_string()));

        let not_found: StorageResult<i32> = StorageResult::NotFound;
        assert!(not_found.is_not_found());

        let already_exists: StorageResult<i32> = StorageResult::AlreadyExists;
        assert!(already_exists.is_already_exists());
    }

    #[test]
    fn test_storage_filter() {
        let filter = StorageFilter::new()
            .filename_pattern("test".to_string())
            .mime_type("text/plain".to_string())
            .min_size(100)
            .max_size(10000)
            .protected(true)
            .limit(10)
            .offset(0);

        let matching_manifest = Manifest::new("QmTest".to_string())
            .filename("test_file.txt".to_string())
            .mimetype("text/plain".to_string())
            .dataset_size(1024)
            .protected(true);

        assert!(filter.matches(&matching_manifest));

        let non_matching_manifest = Manifest::new("QmTest2".to_string())
            .filename("other_file.txt".to_string())
            .mimetype("application/json".to_string())
            .dataset_size(50)
            .protected(false);

        assert!(!filter.matches(&non_matching_manifest));
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = Manifest::new("QmTest".to_string())
            .tree_cid("QmTree".to_string())
            .dataset_size(1024)
            .block_size(256)
            .filename("test.txt".to_string())
            .mimetype("text/plain".to_string())
            .protected(true);

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.tree_cid, deserialized.tree_cid);
        assert_eq!(manifest.dataset_size, deserialized.dataset_size);
        assert_eq!(manifest.block_size, deserialized.block_size);
        assert_eq!(manifest.filename, deserialized.filename);
        assert_eq!(manifest.mimetype, deserialized.mimetype);
        assert_eq!(manifest.protected, deserialized.protected);
        // CID is skipped in serialization
        assert_eq!(deserialized.cid, "");
    }

    #[test]
    fn test_space_serialization() {
        let space = Space::new()
            .total_blocks(10)
            .quota_max_bytes(1000)
            .quota_used_bytes(800)
            .quota_reserved_bytes(100);

        let json = serde_json::to_string(&space).unwrap();
        let deserialized: Space = serde_json::from_str(&json).unwrap();

        assert_eq!(space.total_blocks, deserialized.total_blocks);
        assert_eq!(space.quota_max_bytes, deserialized.quota_max_bytes);
        assert_eq!(space.quota_used_bytes, deserialized.quota_used_bytes);
        assert_eq!(
            space.quota_reserved_bytes,
            deserialized.quota_reserved_bytes
        );
    }
}
