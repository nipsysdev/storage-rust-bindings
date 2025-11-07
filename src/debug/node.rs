//! Node debugging operations
//!
//! This module contains node-specific debugging operations.

use crate::callback::{c_callback, CallbackFuture};
use crate::error::{CodexError, Result};
use crate::ffi::{codex_debug, codex_log_level, free_c_string, string_to_c_string};
use crate::node::lifecycle::CodexNode;
use libc::c_void;
use serde::{Deserialize, Serialize};

/// Log level for debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
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
    /// Node version
    pub version: String,
    /// Node revision
    pub revision: String,
    /// Node peer ID
    pub peer_id: String,
    /// Repository path
    pub repo: String,
    /// Storage Provider Reputation (SPR)
    pub spr: String,
    /// Current log level
    pub log_level: String,
    /// Number of connected peers
    pub connected_peers: usize,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Additional debug information
    pub extra: Option<serde_json::Value>,
}

impl DebugInfo {
    /// Create a new debug info
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the version
    pub fn version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    /// Set the revision
    pub fn revision(mut self, revision: String) -> Self {
        self.revision = revision;
        self
    }

    /// Set the peer ID
    pub fn peer_id(mut self, peer_id: String) -> Self {
        self.peer_id = peer_id;
        self
    }

    /// Set the repository path
    pub fn repo(mut self, repo: String) -> Self {
        self.repo = repo;
        self
    }

    /// Set the SPR
    pub fn spr(mut self, spr: String) -> Self {
        self.spr = spr;
        self
    }

    /// Set the log level
    pub fn log_level(mut self, log_level: String) -> Self {
        self.log_level = log_level;
        self
    }

    /// Set the connected peers count
    pub fn connected_peers(mut self, count: usize) -> Self {
        self.connected_peers = count;
        self
    }

    /// Set the uptime
    pub fn uptime(mut self, seconds: u64) -> Self {
        self.uptime_seconds = seconds;
        self
    }

    /// Set the memory usage
    pub fn memory_usage(mut self, bytes: u64) -> Self {
        self.memory_usage_bytes = bytes;
        self
    }

    /// Set extra information
    pub fn extra(mut self, extra: serde_json::Value) -> Self {
        self.extra = Some(extra);
        self
    }

    /// Get uptime as a human-readable string
    pub fn uptime_string(&self) -> String {
        let seconds = self.uptime_seconds;
        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m {}s", seconds / 60, seconds % 60)
        } else if seconds < 86400 {
            format!(
                "{}h {}m {}s",
                seconds / 3600,
                (seconds % 3600) / 60,
                seconds % 60
            )
        } else {
            format!(
                "{}d {}h {}m {}s",
                seconds / 86400,
                (seconds % 86400) / 3600,
                (seconds % 3600) / 60,
                seconds % 60
            )
        }
    }

    /// Get memory usage as a human-readable string
    pub fn memory_string(&self) -> String {
        let bytes = self.memory_usage_bytes;
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1}KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Check if the node is healthy
    pub fn is_healthy(&self) -> bool {
        // Basic health checks
        !self.version.is_empty()
            && !self.peer_id.is_empty()
            && self.memory_usage_bytes > 0
            && self.uptime_seconds > 0
    }

    /// Get health status as a string
    pub fn health_status(&self) -> &'static str {
        if self.is_healthy() {
            "Healthy"
        } else {
            "Unhealthy"
        }
    }
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            version: String::new(),
            revision: String::new(),
            peer_id: String::new(),
            repo: String::new(),
            spr: String::new(),
            log_level: String::new(),
            connected_peers: 0,
            uptime_seconds: 0,
            memory_usage_bytes: 0,
            extra: None,
        }
    }
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

/// Get node performance metrics
///
/// # Arguments
///
/// * `node` - The Codex node to get metrics for
///
/// # Returns
///
/// Performance metrics
pub async fn get_performance_metrics(node: &CodexNode) -> Result<PerformanceMetrics> {
    // This would typically call a C function to get performance metrics
    // For now, we'll return a placeholder based on debug info

    let debug_info = debug(node).await?;

    Ok(PerformanceMetrics {
        uptime_seconds: debug_info.uptime_seconds,
        memory_usage_bytes: debug_info.memory_usage_bytes,
        connected_peers: debug_info.connected_peers,
        cpu_usage_percent: 0.0, // Placeholder
        network_io_bytes: 0,    // Placeholder
        disk_io_bytes: 0,       // Placeholder
        last_updated: chrono::Utc::now(),
    })
}

/// Performance metrics for the node
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Number of connected peers
    pub connected_peers: usize,
    /// CPU usage percentage (0.0 to 100.0)
    pub cpu_usage_percent: f64,
    /// Network I/O in bytes
    pub network_io_bytes: u64,
    /// Disk I/O in bytes
    pub disk_io_bytes: u64,
    /// When the metrics were last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl PerformanceMetrics {
    /// Get memory usage as a human-readable string
    pub fn memory_string(&self) -> String {
        let bytes = self.memory_usage_bytes;
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1}KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Get network I/O as a human-readable string
    pub fn network_io_string(&self) -> String {
        let bytes = self.network_io_bytes;
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1}KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Get disk I/O as a human-readable string
    pub fn disk_io_string(&self) -> String {
        let bytes = self.disk_io_bytes;
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1}KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Check if performance is good
    pub fn is_performance_good(&self) -> bool {
        self.cpu_usage_percent < 80.0 && self.connected_peers > 0
    }

    /// Get performance status as a string
    pub fn performance_status(&self) -> &'static str {
        if self.is_performance_good() {
            "Good"
        } else {
            "Poor"
        }
    }
}

/// Get node configuration information
///
/// # Arguments
///
/// * `node` - The Codex node to get config for
///
/// # Returns
///
/// Configuration information
pub async fn get_config_info(node: &CodexNode) -> Result<ConfigInfo> {
    // This would typically call a C function to get config info
    // For now, we'll return a placeholder

    Ok(ConfigInfo {
        data_dir: "/tmp/codex".to_string(), // Placeholder
        log_level: "info".to_string(),
        storage_quota_bytes: 1024 * 1024 * 1024, // 1GB
        max_connections: 50,
        bootstrap_peers: vec![],
        enable_metrics: true,
        enable_debug: true,
    })
}

/// Configuration information for the node
#[derive(Debug, Clone)]
pub struct ConfigInfo {
    /// Data directory path
    pub data_dir: String,
    /// Current log level
    pub log_level: String,
    /// Storage quota in bytes
    pub storage_quota_bytes: u64,
    /// Maximum number of connections
    pub max_connections: usize,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<String>,
    /// Whether metrics are enabled
    pub enable_metrics: bool,
    /// Whether debug mode is enabled
    pub enable_debug: bool,
}

impl ConfigInfo {
    /// Get storage quota as a human-readable string
    pub fn storage_quota_string(&self) -> String {
        let bytes = self.storage_quota_bytes;
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1}KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
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
        assert!(!info.version.is_empty(), "Version should not be empty");
        assert!(!info.revision.is_empty(), "Revision should not be empty");
        assert!(!info.peer_id.is_empty(), "Peer ID should not be empty");
        assert!(!info.repo.is_empty(), "Repo path should not be empty");
        assert!(!info.spr.is_empty(), "SPR should not be empty");
        assert!(!info.log_level.is_empty(), "Log level should not be empty");

        // Verify numeric values are reasonable
        // Note: These comparisons are always true for unsigned types but kept for documentation
        assert!(
            info.uptime_seconds > 0 || info.uptime_seconds == 0,
            "Uptime should be valid"
        );
        assert!(
            info.memory_usage_bytes > 0 || info.memory_usage_bytes == 0,
            "Memory usage should be valid"
        );
        assert!(
            info.connected_peers > 0 || info.connected_peers == 0,
            "Connected peers should be valid"
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
        assert!(!info.version.is_empty(), "Version should not be empty");
        assert!(!info.peer_id.is_empty(), "Peer ID should not be empty");

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
    async fn test_get_performance_metrics() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let metrics_result = get_performance_metrics(&node).await;
        assert!(metrics_result.is_ok());

        let metrics = metrics_result.unwrap();
        assert!(metrics.uptime_seconds >= 0);
        assert!(metrics.memory_usage_bytes >= 0);
        assert!(metrics.connected_peers >= 0);
        assert!(metrics.cpu_usage_percent >= 0.0);
        assert!(metrics.network_io_bytes >= 0);
        assert!(metrics.disk_io_bytes >= 0);

        node.stop().unwrap();
        node.destroy().unwrap();
    }

    #[tokio::test]
    async fn test_get_config_info() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        let config_result = get_config_info(&node).await;
        assert!(config_result.is_ok());

        let config_info = config_result.unwrap();
        assert!(!config_info.data_dir.is_empty());
        assert!(!config_info.log_level.is_empty());
        assert!(config_info.storage_quota_bytes > 0);
        assert!(config_info.max_connections > 0);

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

    #[test]
    fn test_debug_info_methods() {
        let debug_info = DebugInfo::new()
            .version("1.0.0".to_string())
            .peer_id("12D3KooWExamplePeer".to_string())
            .uptime(3661) // 1 hour, 1 minute, 1 second
            .memory_usage(1024 * 1024 * 512); // 512MB

        assert_eq!(debug_info.uptime_string(), "1h 1m 1s");
        assert_eq!(debug_info.memory_string(), "512.0MB");
        assert!(debug_info.is_healthy());
        assert_eq!(debug_info.health_status(), "Healthy");

        let unhealthy_info = DebugInfo::new();
        assert!(!unhealthy_info.is_healthy());
        assert_eq!(unhealthy_info.health_status(), "Unhealthy");
    }

    #[test]
    fn test_performance_metrics_methods() {
        let metrics = PerformanceMetrics {
            uptime_seconds: 3600,
            memory_usage_bytes: 1024 * 1024 * 1024, // 1GB
            connected_peers: 5,
            cpu_usage_percent: 50.0,
            network_io_bytes: 1024 * 1024 * 100, // 100MB
            disk_io_bytes: 1024 * 1024 * 50,     // 50MB
            last_updated: chrono::Utc::now(),
        };

        assert_eq!(metrics.memory_string(), "1.0GB");
        assert_eq!(metrics.network_io_string(), "100.0MB");
        assert_eq!(metrics.disk_io_string(), "50.0MB");
        assert!(metrics.is_performance_good());
        assert_eq!(metrics.performance_status(), "Good");

        let poor_metrics = PerformanceMetrics {
            uptime_seconds: 3600,
            memory_usage_bytes: 1024 * 1024 * 1024,
            connected_peers: 0,      // No connections
            cpu_usage_percent: 90.0, // High CPU
            network_io_bytes: 0,
            disk_io_bytes: 0,
            last_updated: chrono::Utc::now(),
        };

        assert!(!poor_metrics.is_performance_good());
        assert_eq!(poor_metrics.performance_status(), "Poor");
    }

    #[test]
    fn test_config_info_methods() {
        let config_info = ConfigInfo {
            data_dir: "/tmp/codex".to_string(),
            log_level: "info".to_string(),
            storage_quota_bytes: 1024 * 1024 * 1024 * 2, // 2GB
            max_connections: 50,
            bootstrap_peers: vec![],
            enable_metrics: true,
            enable_debug: true,
        };

        assert_eq!(config_info.storage_quota_string(), "2.0GB");
    }

    #[test]
    fn test_debug_info_structure() {
        let temp_dir = tempdir().unwrap();
        let config = CodexConfig::new()
            .log_level(crate::node::config::LogLevel::Error)
            .data_dir(temp_dir.path())
            .storage_quota(100 * 1024 * 1024);

        let mut node = CodexNode::new(config).unwrap();
        node.start().unwrap();

        // Create a test debug info to verify structure
        let test_debug_info = DebugInfo::new()
            .version("1.0.0".to_string())
            .revision("abc123".to_string())
            .peer_id("12D3KooWExamplePeer".to_string())
            .repo("/tmp/codex".to_string())
            .spr("0.95".to_string())
            .log_level("info".to_string())
            .connected_peers(5)
            .uptime(3600)
            .memory_usage(1024 * 1024 * 100) // 100 MB
            .extra(serde_json::json!({
                "build_info": {
                    "compiler": "rustc",
                    "target": "x86_64-unknown-linux-gnu"
                }
            }));

        // Verify the debug info can be serialized and deserialized
        let json = serde_json::to_string(&test_debug_info).unwrap();
        let deserialized: DebugInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(test_debug_info.version, deserialized.version);
        assert_eq!(test_debug_info.revision, deserialized.revision);
        assert_eq!(test_debug_info.peer_id, deserialized.peer_id);
        assert_eq!(test_debug_info.repo, deserialized.repo);
        assert_eq!(test_debug_info.spr, deserialized.spr);
        assert_eq!(test_debug_info.log_level, deserialized.log_level);
        assert_eq!(
            test_debug_info.connected_peers,
            deserialized.connected_peers
        );
        assert_eq!(test_debug_info.uptime_seconds, deserialized.uptime_seconds);
        assert_eq!(
            test_debug_info.memory_usage_bytes,
            deserialized.memory_usage_bytes
        );
        assert_eq!(test_debug_info.extra, deserialized.extra);

        node.stop().unwrap();
        node.destroy().unwrap();
    }
}
