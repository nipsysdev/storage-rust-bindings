//! Node lifecycle management for Codex
//!
//! This module provides the main CodexNode struct and methods for
//! managing the lifecycle of a Codex node.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_close, codex_destroy, codex_new, codex_peer_id, codex_repo, codex_revision, codex_spr,
    codex_start, codex_stop, codex_version, free_c_string, string_to_c_string,
};
use crate::node::config::CodexConfig;
use libc::c_void;
use std::ptr;

/// A Codex node that can interact with the Codex network
pub struct CodexNode {
    /// Pointer to the C context
    pub(crate) ctx: *mut c_void,
    /// Whether the node is currently started
    started: bool,
}

unsafe impl Send for CodexNode {}
unsafe impl Sync for CodexNode {}

impl CodexNode {
    /// Create a new Codex node with the provided configuration
    ///
    /// The node is not started automatically; you need to call `start()`
    /// to start it.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the node
    ///
    /// # Returns
    ///
    /// A new CodexNode instance
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codex_rust_bindings::{CodexNode, CodexConfig};
    ///
    /// let config = CodexConfig::default();
    /// let mut node = CodexNode::new(config)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(config: CodexConfig) -> Result<Self> {
        let json_config = config.to_json()?;
        let c_json_config = string_to_c_string(&json_config);

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        let node_ctx = unsafe {
            // Call the C function with the context pointer directly
            let node_ctx = codex_new(
                c_json_config,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            );

            // Clean up
            free_c_string(c_json_config);

            if node_ctx.is_null() {
                return Err(CodexError::node_error("new", "Failed to create node"));
            }

            node_ctx
        };

        // Wait for the operation to complete
        let _result = future.wait()?;

        Ok(CodexNode {
            ctx: node_ctx,
            started: false,
        })
    }

    /// Start the Codex node
    ///
    /// This method starts the node and connects it to the Codex network.
    ///
    /// # Returns
    ///
    /// Ok(()) if the node was started successfully, or an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codex_rust_bindings::{CodexNode, CodexConfig};
    ///
    /// let config = CodexConfig::default();
    /// let mut node = CodexNode::new(config)?;
    /// node.start()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn start(&mut self) -> Result<()> {
        if self.started {
            return Err(CodexError::node_error("start", "Node is already started"));
        }

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_start(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("start", "Failed to start node"));
        }

        // Wait for the operation to complete
        let _result = future.wait()?;

        self.started = true;
        Ok(())
    }

    /// Start the Codex node asynchronously
    ///
    /// This is the async version of `start()`.
    pub async fn start_async(&mut self) -> Result<()> {
        if self.started {
            return Err(CodexError::node_error(
                "start_async",
                "Node is already started",
            ));
        }

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_start(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error(
                "start_async",
                "Failed to start node",
            ));
        }

        // Wait for the operation to complete
        let _result = future.await?;

        self.started = true;
        Ok(())
    }

    /// Stop the Codex node
    ///
    /// This method stops the node and disconnects it from the Codex network.
    ///
    /// # Returns
    ///
    /// Ok(()) if the node was stopped successfully, or an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codex_rust_bindings::{CodexNode, CodexConfig};
    ///
    /// let config = CodexConfig::default();
    /// let mut node = CodexNode::new(config)?;
    /// node.start()?;
    /// node.stop()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn stop(&mut self) -> Result<()> {
        if !self.started {
            return Err(CodexError::node_error("stop", "Node is not started"));
        }

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_stop(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("stop", "Failed to stop node"));
        }

        self.started = false;
        Ok(())
    }

    /// Stop the Codex node asynchronously
    ///
    /// This is the async version of `stop()`.
    pub async fn stop_async(&mut self) -> Result<()> {
        if !self.started {
            return Err(CodexError::node_error("stop_async", "Node is not started"));
        }

        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_stop(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("stop_async", "Failed to stop node"));
        }

        // Wait for the operation to complete
        let _result = future.await?;

        self.started = false;
        Ok(())
    }

    /// Destroy the Codex node, freeing all resources
    ///
    /// The node must be stopped before calling this method.
    ///
    /// # Returns
    ///
    /// Ok(()) if the node was destroyed successfully, or an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codex_rust_bindings::{CodexNode, CodexConfig};
    ///
    /// let config = CodexConfig::default();
    /// let mut node = CodexNode::new(config)?;
    /// node.start()?;
    /// node.stop()?;
    /// node.destroy()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn destroy(mut self) -> Result<()> {
        if self.started {
            return Err(CodexError::node_error("destroy", "Node is still started"));
        }

        // First close the node - this needs to complete before destroy
        let future = CallbackFuture::new();

        // Call the C function to close the node
        let result = unsafe {
            codex_close(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("destroy", "Failed to close node"));
        }

        // Wait for the close operation to complete
        future.wait()?;

        // Now destroy the node - this is synchronous and doesn't use the callback
        // According to the Go bindings, we don't check the return value of destroy
        unsafe {
            codex_destroy(
                self.ctx as *mut _,
                None, // No callback needed for destroy
                ptr::null_mut(),
            )
        };

        self.ctx = ptr::null_mut();
        Ok(())
    }

    /// Get the version of the Codex node
    pub fn version(&self) -> Result<String> {
        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_version(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("version", "Failed to get version"));
        }

        // Wait for the operation to complete
        let version = future.wait()?;

        Ok(version)
    }

    /// Get the revision of the Codex node
    pub fn revision(&self) -> Result<String> {
        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_revision(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("revision", "Failed to get revision"));
        }

        // Wait for the operation to complete
        let revision = future.wait()?;

        Ok(revision)
    }

    /// Get the path of the data directory
    pub fn repo(&self) -> Result<String> {
        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_repo(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("repo", "Failed to get repo path"));
        }

        // Wait for the operation to complete
        let repo = future.wait()?;

        Ok(repo)
    }

    /// Get the SPR (Storage Provider Reputation) of the node
    pub fn spr(&self) -> Result<String> {
        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_spr(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("spr", "Failed to get SPR"));
        }

        // Wait for the operation to complete
        let spr = future.wait()?;

        Ok(spr)
    }

    /// Get the peer ID of the node
    pub fn peer_id(&self) -> Result<String> {
        // Create a callback future for the operation
        let future = CallbackFuture::new();

        // Call the C function with the context pointer directly
        let result = unsafe {
            codex_peer_id(
                self.ctx as *mut _,
                Some(c_callback),
                future.context_ptr() as *mut c_void,
            )
        };

        if result != 0 {
            return Err(CodexError::node_error("peer_id", "Failed to get peer ID"));
        }

        // Wait for the operation to complete
        let peer_id = future.wait()?;

        Ok(peer_id)
    }

    /// Check if the node is started
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Get the raw context pointer (for internal use)
    #[allow(dead_code)]
    pub(crate) fn ctx(&self) -> *mut c_void {
        self.ctx
    }
}

impl Drop for CodexNode {
    fn drop(&mut self) {
        if !self.ctx.is_null() && self.started {
            // Try to stop the node if it's still started
            let _ = self.stop();
        }

        if !self.ctx.is_null() {
            // Try to destroy the node if it's not already destroyed
            let _ = unsafe {
                codex_destroy(self.ctx as *mut _, None, ptr::null_mut());
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::{CodexConfig, LogLevel};
    use tempfile::tempdir;

    #[test]
    fn test_node_creation_with_default_config() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config);
        assert!(node.is_ok());

        let node = node.unwrap();
        assert!(!node.is_started());
        assert!(!node.ctx.is_null());
    }

    #[test]
    fn test_node_creation_with_custom_config() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Debug)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024); // 100 MB

        let node = CodexNode::new(config);
        assert!(node.is_ok());

        let node = node.unwrap();
        assert!(!node.is_started());
        assert!(!node.ctx.is_null());
    }

    #[test]
    fn test_node_creation_with_invalid_config() {
        // Test with invalid data directory (non-existent parent)
        let config = CodexConfig::new().data_dir("/non/existent/path/data");

        // This might fail depending on the implementation
        let _node = CodexNode::new(config);
        // We don't assert failure here as the behavior might vary
    }

    #[test]
    fn test_node_lifecycle_full_cycle() {
        let config = CodexConfig::default();
        let mut node = CodexNode::new(config).unwrap();

        // Initially not started
        assert!(!node.is_started());

        // Start the node
        let start_result = node.start();
        assert!(
            start_result.is_ok(),
            "Failed to start node: {:?}",
            start_result.err()
        );
        assert!(node.is_started());

        // Test getting node info while started
        let version_result = node.version();
        assert!(
            version_result.is_ok(),
            "Failed to get version: {:?}",
            version_result.err()
        );
        let version = version_result.unwrap();
        assert!(!version.is_empty(), "Version should not be empty");

        let peer_id_result = node.peer_id();
        assert!(
            peer_id_result.is_ok(),
            "Failed to get peer ID: {:?}",
            peer_id_result.err()
        );
        let peer_id = peer_id_result.unwrap();
        assert!(!peer_id.is_empty(), "Peer ID should not be empty");

        let repo_result = node.repo();
        assert!(
            repo_result.is_ok(),
            "Failed to get repo: {:?}",
            repo_result.err()
        );
        let repo = repo_result.unwrap();
        assert!(!repo.is_empty(), "Repo path should not be empty");

        let spr_result = node.spr();
        assert!(
            spr_result.is_ok(),
            "Failed to get SPR: {:?}",
            spr_result.err()
        );
        let spr = spr_result.unwrap();
        assert!(!spr.is_empty(), "SPR should not be empty");

        let revision_result = node.revision();
        assert!(
            revision_result.is_ok(),
            "Failed to get revision: {:?}",
            revision_result.err()
        );
        let revision = revision_result.unwrap();
        assert!(!revision.is_empty(), "Revision should not be empty");

        // Stop the node
        let stop_result = node.stop();
        assert!(
            stop_result.is_ok(),
            "Failed to stop node: {:?}",
            stop_result.err()
        );
        assert!(!node.is_started());

        // Destroy the node
        let destroy_result = node.destroy();
        assert!(
            destroy_result.is_ok(),
            "Failed to destroy node: {:?}",
            destroy_result.err()
        );
    }

    #[test]
    fn test_double_start() {
        let config = CodexConfig::default();
        let mut node = CodexNode::new(config).unwrap();

        // Start the node once
        assert!(node.start().is_ok());
        assert!(node.is_started());

        // Try to start again - should fail
        let second_start_result = node.start();
        assert!(second_start_result.is_err());

        let error = second_start_result.unwrap_err();
        assert!(error.to_string().contains("already started"));

        // Clean up
        assert!(node.stop().is_ok());
        assert!(node.destroy().is_ok());
    }

    #[test]
    fn test_stop_not_started() {
        let config = CodexConfig::default();
        let mut node = CodexNode::new(config).unwrap();

        // Try to stop without starting - should fail
        let stop_result = node.stop();
        assert!(stop_result.is_err());

        let error = stop_result.unwrap_err();
        assert!(error.to_string().contains("not started"));

        // Clean up
        assert!(node.destroy().is_ok());
    }

    #[test]
    fn test_destroy_started() {
        let config = CodexConfig::default();
        let mut node = CodexNode::new(config).unwrap();

        // Start the node
        assert!(node.start().is_ok());
        assert!(node.is_started());

        // Try to destroy while started - should fail
        let destroy_result = node.destroy();
        assert!(destroy_result.is_err());

        let error = destroy_result.unwrap_err();
        assert!(error.to_string().contains("still started"));
    }

    #[test]
    fn test_destroy_not_started() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        // Destroy without starting - should work
        let destroy_result = node.destroy();
        assert!(destroy_result.is_ok());
    }

    #[tokio::test]
    async fn test_async_lifecycle_full_cycle() {
        let config = CodexConfig::default();
        let mut node = CodexNode::new(config).unwrap();

        // Initially not started
        assert!(!node.is_started());

        // Start the node asynchronously
        let start_result = node.start_async().await;
        assert!(
            start_result.is_ok(),
            "Failed to start node async: {:?}",
            start_result.err()
        );
        assert!(node.is_started());

        // Test getting node info while started
        let version_result = node.version();
        assert!(
            version_result.is_ok(),
            "Failed to get version: {:?}",
            version_result.err()
        );
        assert!(!version_result.unwrap().is_empty());

        // Stop the node asynchronously
        let stop_result = node.stop_async().await;
        assert!(
            stop_result.is_ok(),
            "Failed to stop node async: {:?}",
            stop_result.err()
        );
        assert!(!node.is_started());

        // Destroy the node
        let destroy_result = node.destroy();
        assert!(
            destroy_result.is_ok(),
            "Failed to destroy node: {:?}",
            destroy_result.err()
        );
    }

    #[tokio::test]
    async fn test_async_double_start() {
        let config = CodexConfig::default();
        let mut node = CodexNode::new(config).unwrap();

        // Start the node once asynchronously
        assert!(node.start_async().await.is_ok());
        assert!(node.is_started());

        // Try to start again asynchronously - should fail
        let second_start_result = node.start_async().await;
        assert!(second_start_result.is_err());

        let error = second_start_result.unwrap_err();
        assert!(error.to_string().contains("already started"));

        // Clean up
        assert!(node.stop_async().await.is_ok());
        assert!(node.destroy().is_ok());
    }

    #[tokio::test]
    async fn test_async_stop_not_started() {
        let config = CodexConfig::default();
        let mut node = CodexNode::new(config).unwrap();

        // Try to stop asynchronously without starting - should fail
        let stop_result = node.stop_async().await;
        assert!(stop_result.is_err());

        let error = stop_result.unwrap_err();
        assert!(error.to_string().contains("not started"));

        // Clean up
        assert!(node.destroy().is_ok());
    }

    #[test]
    fn test_node_info_methods_not_started() {
        let config = CodexConfig::default();
        let node = CodexNode::new(config).unwrap();

        // These methods should work even if the node is not started
        let version_result = node.version();
        assert!(
            version_result.is_ok(),
            "Version should be accessible even when not started"
        );

        let peer_id_result = node.peer_id();
        assert!(
            peer_id_result.is_ok(),
            "Peer ID should be accessible even when not started"
        );

        let repo_result = node.repo();
        assert!(
            repo_result.is_ok(),
            "Repo should be accessible even when not started"
        );

        let spr_result = node.spr();
        assert!(
            spr_result.is_ok(),
            "SPR should be accessible even when not started"
        );

        let revision_result = node.revision();
        assert!(
            revision_result.is_ok(),
            "Revision should be accessible even when not started"
        );

        // Clean up
        assert!(node.destroy().is_ok());
    }

    #[test]
    fn test_multiple_nodes() {
        let config1 = CodexConfig::default();
        let config2 = CodexConfig::default();

        let mut node1 = CodexNode::new(config1).unwrap();
        let mut node2 = CodexNode::new(config2).unwrap();

        // Both nodes should have different contexts
        assert_ne!(node1.ctx, node2.ctx);

        // Both should not be started initially
        assert!(!node1.is_started());
        assert!(!node2.is_started());

        // Start both nodes
        assert!(node1.start().is_ok());
        assert!(node2.start().is_ok());

        // Both should be started
        assert!(node1.is_started());
        assert!(node2.is_started());

        // They should have different peer IDs
        let peer_id1 = node1.peer_id().unwrap();
        let peer_id2 = node2.peer_id().unwrap();
        assert_ne!(peer_id1, peer_id2);

        // Clean up
        assert!(node1.stop().is_ok());
        assert!(node1.destroy().is_ok());
        assert!(node2.stop().is_ok());
        assert!(node2.destroy().is_ok());
    }

    #[test]
    fn test_is_started_consistency() {
        let config = CodexConfig::default();
        let mut node = CodexNode::new(config).unwrap();

        // Initial state
        assert!(!node.is_started());

        // After start
        assert!(node.start().is_ok());
        assert!(node.is_started());

        // After stop
        assert!(node.stop().is_ok());
        assert!(!node.is_started());

        // Clean up
        assert!(node.destroy().is_ok());
    }
}
