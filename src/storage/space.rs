use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{storage_list, storage_space};
use crate::node::lifecycle::StorageNode;
use libc::c_void;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(skip)]
    pub cid: String,
    #[serde(rename = "treeCid", default)]
    pub tree_cid: String,
    #[serde(rename = "datasetSize")]
    pub dataset_size: usize,
    #[serde(rename = "blockSize")]
    pub block_size: usize,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub mimetype: String,
    #[serde(default)]
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestWithCid {
    pub cid: String,
    pub manifest: Manifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    #[serde(rename = "totalBlocks")]
    pub total_blocks: usize,
    #[serde(rename = "quotaMaxBytes")]
    pub quota_max_bytes: u64,
    #[serde(rename = "quotaUsedBytes")]
    pub quota_used_bytes: u64,
    #[serde(rename = "quotaReservedBytes")]
    pub quota_reserved_bytes: u64,
}

pub async fn manifests(node: &StorageNode) -> Result<Vec<Manifest>> {
    let future = CallbackFuture::new();

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            storage_list(
                ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        })
    });

    if result != 0 {
        return Err(StorageError::storage_operation_error(
            "manifests",
            "Failed to list manifests",
        ));
    }

    let manifests_json = future.await?;

    let manifests_with_cid: Vec<ManifestWithCid> = serde_json::from_str(&manifests_json)
        .map_err(|e| StorageError::library_error(format!("Failed to parse manifests: {}", e)))?;

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

pub async fn space(node: &StorageNode) -> Result<Space> {
    let future = CallbackFuture::new();

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            storage_space(
                ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        })
    });

    if result != 0 {
        return Err(StorageError::storage_operation_error(
            "space",
            "Failed to get storage space",
        ));
    }

    let space_json = future.await?;

    let space: Space = serde_json::from_str(&space_json)
        .map_err(|e| StorageError::library_error(format!("Failed to parse space info: {}", e)))?;

    Ok(space)
}
