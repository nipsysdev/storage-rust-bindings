use crate::callback::{c_callback, with_libstorage_lock, CallbackFuture};
use crate::error::{Result, StorageError};
use crate::ffi::{
    free_c_string, storage_delete, storage_exists, storage_fetch, string_to_c_string,
};
use crate::node::lifecycle::StorageNode;
use libc::c_void;

pub async fn fetch(node: &StorageNode, cid: &str) -> Result<super::types::Manifest> {
    let node = node.clone();
    let cid = cid.to_string();

    tokio::task::spawn_blocking(move || {
        if cid.is_empty() {
            return Err(StorageError::invalid_parameter(
                "cid",
                "CID cannot be empty",
            ));
        }

        let future = CallbackFuture::new();

        let c_cid = string_to_c_string(&cid);

        let result = with_libstorage_lock(|| unsafe {
            node.with_ctx(|ctx| {
                storage_fetch(
                    ctx as *mut _,
                    c_cid,
                    Some(c_callback),
                    future.context_ptr() as *mut c_void,
                )
            })
        });

        unsafe {
            free_c_string(c_cid);
        }

        if result != 0 {
            return Err(StorageError::storage_operation_error(
                "fetch",
                "Failed to fetch manifest",
            ));
        }

        let manifest_json = future.wait()?;

        let manifest: super::types::Manifest = serde_json::from_str(&manifest_json)
            .map_err(|e| StorageError::library_error(format!("Failed to parse manifest: {}", e)))?;

        Ok(manifest)
    })
    .await?
}

pub async fn delete(node: &StorageNode, cid: &str) -> Result<()> {
    let node = node.clone();
    let cid = cid.to_string();

    tokio::task::spawn_blocking(move || {
        if cid.is_empty() {
            return Err(StorageError::invalid_parameter(
                "cid",
                "CID cannot be empty",
            ));
        }

        let future = CallbackFuture::new();

        let c_cid = string_to_c_string(&cid);

        let result = with_libstorage_lock(|| unsafe {
            node.with_ctx(|ctx| {
                storage_delete(
                    ctx as *mut _,
                    c_cid,
                    Some(c_callback),
                    future.context_ptr() as *mut c_void,
                )
            })
        });

        unsafe {
            free_c_string(c_cid);
        }

        if result != 0 {
            return Err(StorageError::storage_operation_error(
                "delete",
                "Failed to delete content",
            ));
        }

        future.wait()?;

        Ok(())
    })
    .await?
}

pub async fn exists(node: &StorageNode, cid: &str) -> Result<bool> {
    let node = node.clone();
    let cid = cid.to_string();

    tokio::task::spawn_blocking(move || {
        if cid.is_empty() {
            return Err(StorageError::invalid_parameter(
                "cid",
                "CID cannot be empty",
            ));
        }

        let future = CallbackFuture::new();

        let c_cid = string_to_c_string(&cid);

        let result = with_libstorage_lock(|| unsafe {
            node.with_ctx(|ctx| {
                storage_exists(
                    ctx as *mut _,
                    c_cid,
                    Some(c_callback),
                    future.context_ptr() as *mut c_void,
                )
            })
        });

        unsafe {
            free_c_string(c_cid);
        }

        if result != 0 {
            return Err(StorageError::storage_operation_error(
                "exists",
                "Failed to check if content exists",
            ));
        }

        let exists_str = future.wait()?;

        let exists = exists_str.parse::<bool>().map_err(|e| {
            StorageError::library_error(format!("Failed to parse exists result: {}", e))
        })?;

        Ok(exists)
    })
    .await?
}
