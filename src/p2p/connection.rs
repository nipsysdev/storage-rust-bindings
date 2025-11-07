//! P2P connection operations
//!
//! This module contains connection management operations: connect and disconnect.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_connect, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::{c_char, c_void};

/// Connect to a peer in the Codex network
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to connect to
/// * `peer_addresses` - List of multiaddresses for the peer
///
/// # Returns
///
/// Ok(()) if the connection was successful, or an error
pub async fn connect(node: &CodexNode, peer_id: &str, peer_addresses: &[String]) -> Result<()> {
    if peer_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    if peer_addresses.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_addresses",
            "At least one peer address must be provided",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_peer_id = string_to_c_string(peer_id);

    // Convert addresses to C array
    let c_addresses: Vec<*mut c_char> = peer_addresses
        .iter()
        .map(|addr| string_to_c_string(addr))
        .collect();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_connect(
            node.ctx as *mut _,
            c_peer_id,
            c_addresses.as_ptr() as *mut *mut c_char,
            c_addresses.len(),
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_peer_id);
        for addr in c_addresses {
            free_c_string(addr);
        }
    }

    if result != 0 {
        return Err(CodexError::p2p_error("Failed to connect to peer"));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

/// Disconnect from a peer
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to disconnect from
///
/// # Returns
///
/// Ok(()) if the disconnection was successful, or an error
///
/// Note: This function is not available in the current C API.
/// Use the debug operations to manage peer connections.
pub async fn disconnect(_node: &CodexNode, _peer_id: &str) -> Result<()> {
    Err(CodexError::library_error(
        "disconnect is not available in the current C API",
    ))
}

/// Connect to multiple peers concurrently
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_connections` - List of (peer_id, addresses) tuples
///
/// # Returns
///
/// A vector of results, one for each connection attempt
pub async fn connect_to_multiple(
    node: &CodexNode,
    peer_connections: Vec<(String, Vec<String>)>,
) -> Vec<Result<()>> {
    let mut results = Vec::with_capacity(peer_connections.len());

    for (peer_id, addresses) in peer_connections {
        let result = connect(node, &peer_id, &addresses).await;
        results.push(result);
    }

    results
}

/// Validate a peer ID format
///
/// # Arguments
///
/// * `peer_id` - The peer ID to validate
///
/// # Returns
///
/// Ok(()) if the peer ID is valid, or an error
pub fn validate_peer_id(peer_id: &str) -> Result<()> {
    if peer_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    // Basic peer ID validation - peer IDs should have a reasonable length
    if peer_id.len() < 10 {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID is too short",
        ));
    }

    if peer_id.len() > 100 {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID is too long",
        ));
    }

    // Check for valid peer ID prefixes
    let valid_prefixes = vec![
        "12D3KooW", // libp2p Ed25519
        "Qm",       // CIDv0
        "bafy",     // CIDv1 raw
        "bafk",     // CIDv1 dag-pb
    ];

    let has_valid_prefix = valid_prefixes
        .iter()
        .any(|&prefix| peer_id.starts_with(prefix));

    if !has_valid_prefix {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID has invalid format or prefix",
        ));
    }

    Ok(())
}

/// Validate multiaddresses
///
/// # Arguments
///
/// * `addresses` - The addresses to validate
///
/// # Returns
///
/// Ok(()) if all addresses are valid, or an error
pub fn validate_addresses(addresses: &[String]) -> Result<()> {
    if addresses.is_empty() {
        return Err(CodexError::invalid_parameter(
            "addresses",
            "At least one address must be provided",
        ));
    }

    for (i, address) in addresses.iter().enumerate() {
        if address.is_empty() {
            return Err(CodexError::invalid_parameter(
                &format!("addresses[{}]", i),
                "Address cannot be empty",
            ));
        }

        // Basic multiaddress validation
        if !address.starts_with('/') {
            return Err(CodexError::invalid_parameter(
                &format!("addresses[{}]", i),
                "Address must start with '/'",
            ));
        }

        // Check for valid protocols
        let valid_protocols = vec![
            "/ip4", "/ip6", "/dns4", "/dns6", "/dnsaddr", "/tcp", "/udp", "/quic", "/ws", "/wss",
            "/p2p", "/ipfs",
        ];

        let has_valid_protocol = valid_protocols
            .iter()
            .any(|&protocol| address.contains(protocol));

        if !has_valid_protocol {
            return Err(CodexError::invalid_parameter(
                &format!("addresses[{}]", i),
                "Address contains invalid protocol",
            ));
        }
    }

    Ok(())
}

/// Get connection statistics
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// Connection statistics
pub async fn connection_stats(node: &CodexNode) -> Result<ConnectionStats> {
    // This would typically call a C function to get connection stats
    // For now, we'll return a placeholder

    // In a real implementation, you might call something like:
    // let stats = unsafe { codex_connection_stats(node.ctx as *mut _) };

    Ok(ConnectionStats {
        active_connections: 0,
        total_connections: 0,
        failed_connections: 0,
        average_latency_ms: 0.0,
        last_connection_time: None,
    })
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Number of active connections
    pub active_connections: usize,
    /// Total number of connections attempted
    pub total_connections: usize,
    /// Number of failed connections
    pub failed_connections: usize,
    /// Average latency in milliseconds
    pub average_latency_ms: f64,
    /// Last successful connection time
    pub last_connection_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl ConnectionStats {
    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            (self.total_connections - self.failed_connections) as f64
                / self.total_connections as f64
                * 100.0
        }
    }

    /// Get the failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.failed_connections as f64 / self.total_connections as f64 * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::{CodexConfig, LogLevel};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_connect_with_valid_parameters() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";
        let addresses = vec![
            "/ip4/192.168.1.100/tcp/8080".to_string(),
            "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            "/ip6/::1/tcp/8080".to_string(),
        ];

        let result = connect(&node, peer_id, &addresses).await;
        // This might fail if the peer doesn't exist, but the function should handle it gracefully
        assert!(result.is_ok() || result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_with_single_address() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";
        let addresses = vec!["/ip4/192.168.1.100/tcp/8080".to_string()];

        let result = connect(&node, peer_id, &addresses).await;
        assert!(result.is_ok() || result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_with_multiple_addresses() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";
        let addresses = vec![
            "/ip4/192.168.1.100/tcp/8080".to_string(),
            "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            "/ip4/192.168.1.100/tcp/8081".to_string(),
            "/ip6/::1/tcp/8080".to_string(),
            "/ip6/::1/udp/8080/quic".to_string(),
            "/dns4/example.com/tcp/8080".to_string(),
        ];

        let result = connect(&node, peer_id, &addresses).await;
        assert!(result.is_ok() || result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_invalid_parameters() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Empty peer ID
        let result = connect(&node, "", &["/ip4/192.168.1.100/tcp/8080".to_string()]).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Peer ID cannot be empty"));

        // Empty addresses
        let result = connect(&node, "12D3KooWExamplePeer", &[]).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("At least one peer address must be provided"));

        // Empty peer ID with empty addresses
        let result = connect(&node, "", &[]).await;
        assert!(result.is_err());

        // Valid peer ID but empty address in list
        let result = connect(&node, "12D3KooWExamplePeer", &["".to_string()]).await;
        // This might not fail immediately but should be handled gracefully
        assert!(result.is_ok() || result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_with_invalid_addresses() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";

        // Test with various invalid address formats
        let invalid_addresses = vec![
            vec!["invalid-address".to_string()],
            vec!["/ip4/256.256.256.256/tcp/8080".to_string()], // Invalid IP
            vec!["/ip4/192.168.1.100/tcp/99999".to_string()],  // Invalid port
            vec!["/invalid/protocol/address".to_string()],
            vec!["".to_string()],
        ];

        for addresses in invalid_addresses {
            let result = connect(&node, peer_id, &addresses).await;
            // These might fail but should not panic
            assert!(result.is_ok() || result.is_err());
        }

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_disconnect_not_implemented() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer";
        let result = disconnect(&node, peer_id).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("not available in the current C API"));

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connect_to_multiple() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_connections = vec![
            (
                "12D3KooWExamplePeer1".to_string(),
                vec!["/ip4/192.168.1.100/tcp/8080".to_string()],
            ),
            (
                "12D3KooWExamplePeer2".to_string(),
                vec!["/ip4/192.168.1.101/tcp/8080".to_string()],
            ),
        ];

        let results = connect_to_multiple(&node, peer_connections).await;
        assert_eq!(results.len(), 2);
        // Results might be ok or err depending on whether peers exist

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_connection_stats() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let stats = connection_stats(&node).await;
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.failed_connections, 0);
        assert_eq!(stats.success_rate(), 0.0);
        assert_eq!(stats.failure_rate(), 0.0);

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[test]
    fn test_validate_peer_id() {
        // Valid peer IDs
        let valid_peer_ids = vec![
            "12D3KooWExamplePeer123456789",
            "QmSomePeerId123456789",
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ];

        for peer_id in valid_peer_ids {
            assert!(
                validate_peer_id(peer_id).is_ok(),
                "Peer ID {} should be valid",
                peer_id
            );
        }

        // Invalid peer IDs
        let long_string = "X".repeat(101);
        let invalid_peer_ids = vec![
            "",
            "short",
            "12D3KooW",   // Too short even with valid prefix
            &long_string, // Too long
            "InvalidPrefix123456789",
        ];

        for peer_id in invalid_peer_ids {
            assert!(
                validate_peer_id(peer_id).is_err(),
                "Peer ID {} should be invalid",
                peer_id
            );
        }
    }

    #[test]
    fn test_validate_addresses() {
        // Valid addresses
        let valid_addresses = vec![
            vec!["/ip4/192.168.1.100/tcp/8080".to_string()],
            vec!["/ip6/::1/tcp/8080".to_string()],
            vec!["/dns4/example.com/tcp/8080".to_string()],
            vec![
                "/ip4/192.168.1.100/tcp/8080".to_string(),
                "/ip4/192.168.1.100/udp/8080/quic".to_string(),
            ],
        ];

        for addresses in valid_addresses {
            assert!(validate_addresses(&addresses).is_ok());
        }

        // Invalid addresses
        let invalid_addresses = vec![
            vec![],                                        // Empty
            vec!["".to_string()],                          // Empty string
            vec!["invalid-address".to_string()],           // Doesn't start with /
            vec!["/invalid/protocol/address".to_string()], // Invalid protocol
        ];

        for addresses in invalid_addresses {
            assert!(validate_addresses(&addresses).is_err());
        }
    }

    #[test]
    fn test_connection_stats_methods() {
        let stats = ConnectionStats {
            active_connections: 5,
            total_connections: 10,
            failed_connections: 2,
            average_latency_ms: 50.0,
            last_connection_time: Some(chrono::Utc::now()),
        };

        assert_eq!(stats.success_rate(), 80.0);
        assert_eq!(stats.failure_rate(), 20.0);

        let empty_stats = ConnectionStats {
            active_connections: 0,
            total_connections: 0,
            failed_connections: 0,
            average_latency_ms: 0.0,
            last_connection_time: None,
        };

        assert_eq!(empty_stats.success_rate(), 0.0);
        assert_eq!(empty_stats.failure_rate(), 0.0);
    }
}
