//! Types for storage operations

use serde::{Deserialize, Serialize};

/// Manifest information for a stored content
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
            self.dataset_size.div_ceil(self.block_size)
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
            self.filename
                .rfind('.')
                .map(|dot_pos| self.filename[dot_pos + 1..].to_lowercase())
        } else {
            None
        }
    }

    /// Get a human-readable size string
    pub fn size_string(&self) -> String {
        bytesize::ByteSize::b(self.dataset_size as u64).to_string()
    }
}

/// Storage space information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
