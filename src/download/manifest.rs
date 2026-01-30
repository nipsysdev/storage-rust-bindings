use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::download::types::Manifest;
use crate::error::{Result, StorageError};
use crate::ffi::{free_c_string, storage_download_manifest, string_to_c_string};
use crate::node::lifecycle::StorageNode;
use libc::c_void;

pub async fn download_manifest(node: &StorageNode, cid: &str) -> Result<Manifest> {
    if cid.is_empty() {
        return Err(StorageError::invalid_parameter(
            "cid",
            "CID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();

    let context_ptr = future.context_ptr() as *mut c_void;

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            let c_cid = string_to_c_string(cid);
            let result =
                storage_download_manifest(ctx as *mut _, c_cid, Some(c_callback), context_ptr);

            free_c_string(c_cid);

            result
        })
    });

    if result != 0 {
        return Err(StorageError::download_error("Failed to download manifest"));
    }

    let manifest_json = future.await?;

    let manifest: Manifest = serde_json::from_str(&manifest_json)
        .map_err(|e| StorageError::library_error(format!("Failed to parse manifest: {}", e)))?;

    Ok(manifest)
}
