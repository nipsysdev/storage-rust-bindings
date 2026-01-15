use crate::callback::{c_callback, with_libcodex_lock, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{free_c_string, storage_peer_debug, storage_peer_id, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use crate::p2p::types::PeerRecord;
use libc::c_void;

pub async fn get_peer_info(node: &CodexNode, peer_id: &str) -> Result<PeerRecord> {
    let node = node.clone();
    let peer_id = peer_id.to_string();

    tokio::task::spawn_blocking(move || {
        if peer_id.is_empty() {
            return Err(CodexError::invalid_parameter(
                "peer_id",
                "Peer ID cannot be empty",
            ));
        }

        let future = CallbackFuture::new();

        with_libcodex_lock(|| {
            let c_peer_id = string_to_c_string(&peer_id);

            let result = unsafe {
                node.with_ctx(|ctx| {
                    storage_peer_debug(
                        ctx as *mut _,
                        c_peer_id,
                        Some(c_callback),
                        future.context_ptr() as *mut c_void,
                    )
                })
            };

            unsafe {
                free_c_string(c_peer_id);
            }

            if result != 0 {
                return Err(CodexError::p2p_error("Failed to get peer info"));
            }

            Ok(())
        })?;

        let peer_json = future.wait()?;

        let peer: PeerRecord = serde_json::from_str(&peer_json)
            .map_err(|e| CodexError::library_error(format!("Failed to parse peer info: {}", e)))?;

        Ok(peer)
    })
    .await?
}

pub async fn get_peer_id(node: &CodexNode) -> Result<String> {
    let node = node.clone();

    tokio::task::spawn_blocking(move || {
        let future = CallbackFuture::new();

        with_libcodex_lock(|| {
            let result = unsafe {
                node.with_ctx(|ctx| {
                    storage_peer_id(
                        ctx as *mut _,
                        Some(c_callback),
                        future.context_ptr() as *mut c_void,
                    )
                })
            };

            if result != 0 {
                return Err(CodexError::p2p_error("Failed to get peer ID"));
            }

            Ok(())
        })?;

        let peer_id = future.wait()?;

        Ok(peer_id)
    })
    .await?
}
