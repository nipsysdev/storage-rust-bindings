use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{storage_delete, storage_exists, storage_fetch, string_to_c_string};
use crate::node::lifecycle::StorageNode;

pub async fn fetch(node: &StorageNode, cid: &str) -> Result<super::types::Manifest> {
    if cid.is_empty() {
        return Err(StorageError::invalid_parameter(
            "cid",
            "CID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let c_cid = string_to_c_string(cid);

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            storage_fetch(
                ctx as *mut _,
                c_cid.as_ptr(),
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::storage_operation_error(
            "fetch",
            "Failed to fetch manifest",
        ));
    }

    let manifest_json = future.await?;

    let manifest: super::types::Manifest = serde_json::from_str(&manifest_json)
        .map_err(|e| StorageError::library_error(format!("Failed to parse manifest: {}", e)))?;

    Ok(manifest)
}

pub async fn delete(node: &StorageNode, cid: &str) -> Result<()> {
    if cid.is_empty() {
        return Err(StorageError::invalid_parameter(
            "cid",
            "CID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let c_cid = string_to_c_string(cid);

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            storage_delete(
                ctx as *mut _,
                c_cid.as_ptr(),
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::storage_operation_error(
            "delete",
            "Failed to delete content",
        ));
    }

    future.await?;

    Ok(())
}

pub async fn exists(node: &StorageNode, cid: &str) -> Result<bool> {
    if cid.is_empty() {
        return Err(StorageError::invalid_parameter(
            "cid",
            "CID cannot be empty",
        ));
    }

    let future = CallbackFuture::new();
    let context_ptr = future.context_ptr();

    let c_cid = string_to_c_string(cid);

    let result = with_libstorage_lock(|| unsafe {
        node.with_ctx(|ctx| {
            storage_exists(
                ctx as *mut _,
                c_cid.as_ptr(),
                Some(c_callback),
                context_ptr.as_ptr(),
            )
        })
    });

    if result != 0 {
        return Err(StorageError::storage_operation_error(
            "exists",
            "Failed to check if content exists",
        ));
    }

    let exists_str = future.await?;

    let exists = exists_str.parse::<bool>().map_err(|e| {
        StorageError::library_error(format!("Failed to parse exists result: {}", e))
    })?;

    Ok(exists)
}
