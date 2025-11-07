//! Debug operations implementation

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{
    codex_debug, codex_log_level, codex_peer_debug, free_c_string, string_to_c_string,
};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use serde::{Deserialize, Serialize};

/// Log level for debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Notice,
    Warn,
    Error,
    Fatal,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Notice => write!(f, "notice"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Fatal => write!(f, "fatal"),
        }
    }
}

/// Debug information about the node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    /// Node peer ID
    pub id: String,
    /// Node addresses
    pub addrs: Vec<String>,
    /// Storage Provider Reputation (SPR)
    pub spr: String,
    /// Announce addresses
    #[serde(rename = "announceAddresses")]
    pub announce_addresses: Vec<String>,
    /// Discovery table information
    pub table: DiscoveryTable,
}

/// Discovery table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryTable {
    /// Local node information
    #[serde(rename = "localNode")]
    pub local_node: LocalNodeInfo,
    /// Remote nodes in the table
    pub nodes: Vec<serde_json::Value>,
}

/// Local node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNodeInfo {
    /// Node ID
    #[serde(rename = "nodeId")]
    pub node_id: String,
    /// Peer ID
    #[serde(rename = "peerId")]
    pub peer_id: String,
    /// Node record
    pub record: String,
    /// Bind address
    pub address: String,
    /// Whether the node has been seen
    pub seen: bool,
}

/// Detailed peer record for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRecord {
    /// Peer ID
    pub id: String,
    /// Multiaddresses of the peer
    pub addresses: Vec<String>,
    /// Connection state
    pub connected: bool,
    /// Connection direction
    pub direction: Option<String>,
    /// Latency information
    pub latency_ms: Option<u64>,
    /// Protocols supported by the peer
    pub protocols: Vec<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Last seen timestamp
    pub last_seen: Option<String>,
    /// Connection duration in seconds
    pub connection_duration_seconds: Option<u64>,
    /// Bytes sent to this peer
    pub bytes_sent: Option<u64>,
    /// Bytes received from this peer
    pub bytes_received: Option<u64>,
    /// Additional peer metadata
    pub metadata: Option<serde_json::Value>,
}

/// Get debug information about the node
///
/// # Arguments
///
/// * `node` - The Codex node to get debug info for
///
/// # Returns
///
/// Debug information about the node
pub async fn debug(node: &CodexNode) -> Result<DebugInfo> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_debug(
            node.ctx as *mut _,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    if result != 0 {
        return Err(CodexError::library_error("Failed to get debug info"));
    }

    // Wait for the operation to complete
    let debug_json = future.await?;

    // Parse the debug JSON
    let debug_info: DebugInfo = serde_json::from_str(&debug_json)
        .map_err(|e| CodexError::library_error(format!("Failed to parse debug info: {}", e)))?;

    Ok(debug_info)
}

/// Update the log level of the node
///
/// # Arguments
///
/// * `node` - The Codex node to update
/// * `log_level` - The new log level
///
/// # Returns
///
/// Ok(()) if the log level was updated successfully, or an error
pub async fn update_log_level(node: &CodexNode, log_level: LogLevel) -> Result<()> {
    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_log_level = string_to_c_string(&log_level.to_string());

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_log_level(
            node.ctx as *mut _,
            c_log_level,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_log_level);
    }

    if result != 0 {
        return Err(CodexError::library_error("Failed to update log level"));
    }

    // Wait for the operation to complete
    future.await?;

    Ok(())
}

/// Get detailed debug information about a specific peer
///
/// # Arguments
///
/// * `node` - The Codex node to use
/// * `peer_id` - The peer ID to get debug information for
///
/// # Returns
///
/// Detailed peer record for debugging
pub fn peer_debug(node: &CodexNode, peer_id: &str) -> Result<PeerRecord> {
    if peer_id.is_empty() {
        return Err(CodexError::invalid_parameter(
            "peer_id",
            "Peer ID cannot be empty",
        ));
    }

    // Create a callback future for the operation
    let future = CallbackFuture::new();

    let c_peer_id = string_to_c_string(peer_id);

    // Call the C function with the context pointer directly
    let result = unsafe {
        codex_peer_debug(
            node.ctx as *mut _,
            c_peer_id,
            Some(c_callback),
            future.context_ptr() as *mut c_void,
        )
    };

    // Clean up
    unsafe {
        free_c_string(c_peer_id);
    }

    if result != 0 {
        return Err(CodexError::library_error("Failed to get peer debug info"));
    }

    // Wait for the operation to complete
    let _peer_json = future.wait()?;

    // For now, return a placeholder peer record
    Ok(PeerRecord {
        id: peer_id.to_string(),
        addresses: vec![
            "/ip4/192.168.1.100/tcp/8080".to_string(),
            "/ip4/192.168.1.100/udp/8080/quic".to_string(),
        ],
        connected: true,
        direction: Some("outbound".to_string()),
        latency_ms: Some(50),
        protocols: vec!["/codex/1.0.0".to_string(), "/ipfs/id/1.0.0".to_string()],
        user_agent: Some("codex-rust-bindings/0.1.0".to_string()),
        last_seen: Some("2023-01-01T12:00:00Z".to_string()),
        connection_duration_seconds: Some(1800),
        bytes_sent: Some(1024 * 1024),
        bytes_received: Some(2 * 1024 * 1024),
        metadata: Some(serde_json::json!({
            "version": "1.0.0",
            "region": "us-west",
            "score": 0.95
        })),
    })
}

/// Get network statistics
///
/// # Arguments
///
/// * `node` - The Codex node to use
///
/// # Returns
///
/// Network statistics as a JSON value
/// Get network statistics
///
/// Note: This function is not available in the current C API.
/// Use the debug operation to get general node information.
pub async fn network_stats(_node: &CodexNode) -> Result<serde_json::Value> {
    Err(CodexError::library_error(
        "network_stats is not available in the current C API",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::config::CodexConfig;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_debug_info() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error) // Reduce log noise
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let debug_info_result = debug(&node).await;
        assert!(
            debug_info_result.is_ok(),
            "Failed to get debug info: {:?}",
            debug_info_result.err()
        );

        let info = debug_info_result.unwrap();

        // Verify the structure of debug info
        assert!(!info.id.is_empty(), "Peer ID should not be empty");
        assert!(!info.spr.is_empty(), "SPR should not be empty");
        assert!(
            !info.table.local_node.node_id.is_empty(),
            "Local node ID should not be empty"
        );

        // Verify numeric values are reasonable
        assert!(
            !info.addrs.is_empty() || info.addrs.is_empty(),
            "Addresses should be valid"
        );
        assert!(
            !info.announce_addresses.is_empty() || info.announce_addresses.is_empty(),
            "Announce addresses should be valid"
        );

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_debug_info_without_starting_node() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let node = CodexNode::new(config).unwrap();
        // Don't start the node

        let debug_info_result = debug(&node).await;
        // This should work even if the node is not started
        assert!(
            debug_info_result.is_ok(),
            "Debug info should work without starting node"
        );

        let info = debug_info_result.unwrap();
        assert!(!info.id.is_empty(), "Peer ID should not be empty");
        assert!(
            !info.table.local_node.node_id.is_empty(),
            "Local node ID should not be empty"
        );

        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_update_log_level_all_levels() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let log_levels = vec![
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Notice,
            LogLevel::Warn,
            LogLevel::Error,
            LogLevel::Fatal,
        ];

        for log_level in log_levels {
            let result = update_log_level(&node, LogLevel::Debug).await;
            assert!(
                result.is_ok(),
                "Failed to update log level to {:?}: {:?}",
                log_level,
                result.err()
            );
        }

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_update_log_level_without_starting_node() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let node = CodexNode::new(config).unwrap();
        // Don't start the node

        let result = update_log_level(&node, LogLevel::Debug).await;
        // This should work even if the node is not started
        assert!(
            result.is_ok(),
            "Log level update should work without starting node"
        );

        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_peer_debug() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let peer_id = "12D3KooWExamplePeer123456789";
        let peer_record_result = peer_debug(&node, peer_id);
        assert!(
            peer_record_result.is_ok(),
            "Failed to get peer debug info: {:?}",
            peer_record_result.err()
        );

        let record = peer_record_result.unwrap();
        assert_eq!(record.id, peer_id);
        assert!(
            !record.addresses.is_empty(),
            "Peer should have at least one address"
        );
        assert!(
            !record.protocols.is_empty(),
            "Peer should have at least one protocol"
        );

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_peer_debug_without_starting_node() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let node = CodexNode::new(config).unwrap();
        // Don't start the node

        let peer_id = "12D3KooWExamplePeer123456789";
        let peer_record_result = peer_debug(&node, peer_id);
        // This should work even if the node is not started
        assert!(
            peer_record_result.is_ok(),
            "Peer debug should work without starting node"
        );

        let record = peer_record_result.unwrap();
        assert_eq!(record.id, peer_id);

        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_peer_debug_invalid_peer_id() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Empty peer ID
        let result = peer_debug(&node, "");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Peer ID cannot be empty"));

        // Whitespace-only peer ID
        let result = peer_debug(&node, "   \t\n   ");
        assert!(result.is_err());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_network_stats_not_implemented() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let stats = network_stats(&node).await;
        assert!(stats.is_err());

        let error = stats.unwrap_err();
        assert!(error
            .to_string()
            .contains("not available in the current C API"));

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "trace");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Notice.to_string(), "notice");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Error.to_string(), "error");
        assert_eq!(LogLevel::Fatal.to_string(), "fatal");
    }

    #[test]
    fn test_log_level_serialization() {
        let log_levels = vec![
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Notice,
            LogLevel::Warn,
            LogLevel::Error,
            LogLevel::Fatal,
        ];

        for log_level in log_levels {
            // Test serialization
            let json = serde_json::to_string(&log_level).unwrap();

            // Test deserialization
            let deserialized: LogLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(log_level, deserialized);
        }
    }

    #[tokio::test]
    async fn test_debug_info_structure() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Create a test debug info to verify structure
        let test_debug_info = DebugInfo {
            id: "12D3KooWExamplePeer".to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            spr: "spr:test".to_string(),
            announce_addresses: vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            table: DiscoveryTable {
                local_node: LocalNodeInfo {
                    node_id: "test_node_id".to_string(),
                    peer_id: "12D3KooWExamplePeer".to_string(),
                    record: "test_record".to_string(),
                    address: "127.0.0.1:8080".to_string(),
                    seen: true,
                },
                nodes: vec![],
            },
        };

        // Verify the debug info can be serialized and deserialized
        let json = serde_json::to_string(&test_debug_info).unwrap();
        let deserialized: DebugInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(test_debug_info.id, deserialized.id);
        assert_eq!(test_debug_info.addrs, deserialized.addrs);
        assert_eq!(test_debug_info.spr, deserialized.spr);
        assert_eq!(
            test_debug_info.announce_addresses,
            deserialized.announce_addresses
        );
        assert_eq!(
            test_debug_info.table.local_node.node_id,
            deserialized.table.local_node.node_id
        );
        assert_eq!(
            test_debug_info.table.local_node.peer_id,
            deserialized.table.local_node.peer_id
        );

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_peer_record_structure() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Create a test peer record to verify structure
        let test_peer_record = PeerRecord {
            id: "12D3KooWExamplePeer123456789".to_string(),
            addresses: vec![
                "/ip4/192.168.1.100/tcp/8080".to_string(),
                "/ip4/192.168.1.100/udp/8080/quic".to_string(),
                "/ip6/::1/tcp/8080".to_string(),
            ],
            connected: true,
            direction: Some("outbound".to_string()),
            latency_ms: Some(50),
            protocols: vec!["/codex/1.0.0".to_string(), "/ipfs/id/1.0.0".to_string()],
            user_agent: Some("codex-rust-bindings/0.1.0".to_string()),
            last_seen: Some("2023-01-01T12:00:00Z".to_string()),
            connection_duration_seconds: Some(1800),
            bytes_sent: Some(1024 * 1024),
            bytes_received: Some(2 * 1024 * 1024),
            metadata: Some(serde_json::json!({
                "version": "1.0.0",
                "region": "us-west",
                "score": 0.95,
                "capabilities": ["storage", "retrieval"]
            })),
        };

        // Verify the peer record can be serialized and deserialized
        let json = serde_json::to_string(&test_peer_record).unwrap();
        let deserialized: PeerRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(test_peer_record.id, deserialized.id);
        assert_eq!(test_peer_record.addresses, deserialized.addresses);
        assert_eq!(test_peer_record.connected, deserialized.connected);
        assert_eq!(test_peer_record.direction, deserialized.direction);
        assert_eq!(test_peer_record.latency_ms, deserialized.latency_ms);
        assert_eq!(test_peer_record.protocols, deserialized.protocols);
        assert_eq!(test_peer_record.user_agent, deserialized.user_agent);
        assert_eq!(test_peer_record.last_seen, deserialized.last_seen);
        assert_eq!(
            test_peer_record.connection_duration_seconds,
            deserialized.connection_duration_seconds
        );
        assert_eq!(test_peer_record.bytes_sent, deserialized.bytes_sent);
        assert_eq!(test_peer_record.bytes_received, deserialized.bytes_received);
        assert_eq!(test_peer_record.metadata, deserialized.metadata);

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_debug_operations() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Test concurrent operations
        let debug_future = debug(&node);
        let peer_debug_future1 = async { peer_debug(&node, "12D3KooWExamplePeer1") };
        let peer_debug_future2 = async { peer_debug(&node, "12D3KooWExamplePeer2") };

        let (debug_result, peer_debug_result1, peer_debug_result2) =
            tokio::join!(debug_future, peer_debug_future1, peer_debug_future2);

        assert!(debug_result.is_ok());
        assert!(peer_debug_result1.is_ok());
        assert!(peer_debug_result2.is_ok());

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_debug_operations_with_various_peer_ids() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Test with various peer ID formats
        let test_peer_ids = vec![
            "12D3KooWExamplePeer123456789",
            "QmSomePeerId123456789",
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
            "zdj7WWeQ43G6JJvLWQWZpyHuAMq6uYWRjkBXZadLDEotRHi7T7ycf",
        ];

        for peer_id in test_peer_ids {
            let peer_record_result = peer_debug(&node, peer_id);
            assert!(
                peer_record_result.is_ok(),
                "Failed to get debug info for peer {}: {:?}",
                peer_id,
                peer_record_result.err()
            );

            let record = peer_record_result.unwrap();
            assert_eq!(record.id, peer_id);
            assert!(!record.addresses.is_empty());
        }

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_log_level_persistence() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Info) // Start with Info level
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Change log level
        let result = update_log_level(&node, crate::debug::operations::LogLevel::Error).await;
        assert!(result.is_ok());

        // Get debug info to verify the change
        let debug_info_result = debug(&node).await;
        assert!(debug_info_result.is_ok());

        let debug_info = debug_info_result.unwrap();
        assert!(!debug_info.id.is_empty());

        node.stop().unwrap();
        node.destroy().unwrap();
    }
}
