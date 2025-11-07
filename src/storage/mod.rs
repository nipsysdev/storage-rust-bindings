//! Storage management operations for Codex
//!
//! This module provides functionality for managing stored content,
//! including listing, fetching, deleting, and checking existence of content.

pub mod crud;
pub mod space;
pub mod types;

// Re-export CRUD operations
pub use crud::{batch_delete, batch_exists, delete, exists, fetch, validate_cid};

// Re-export space management operations
pub use space::{
    find_manifests_by_filename, find_manifests_by_mime_type, get_optimization_suggestions,
    manifests, space, storage_stats, Manifest, ManifestWithCid, OptimizationSuggestion, Space,
    StorageStats, SuggestionCategory, SuggestionPriority,
};

// Re-export types
pub use types::{Manifest as StorageManifest, StorageFilter, StorageResult};
