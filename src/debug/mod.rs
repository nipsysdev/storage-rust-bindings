//! Debug operations for Codex
//!
//! This module provides functionality for debugging and diagnostics,
//! including getting node info, updating log levels, and peer debugging.

pub mod logging;
pub mod node;
pub mod peer;

// Re-export node debugging operations
pub use node::{
    debug, get_config_info, get_performance_metrics, update_log_level, DebugInfo, LogLevel,
    PerformanceMetrics,
};

// Re-export peer debugging operations
pub use peer::{
    analyze_peer_patterns, get_peer_connection_history, network_stats, peer_debug,
    ConnectionDirection, ConnectionEvent, ConnectionEventType, ConnectionQuality,
    PeerPatternAnalysis, PeerRecord, ReliabilityRating,
};

// Re-export logging operations
pub use logging::{
    analyze_log_patterns, parse_log_line, LogAnalysis, LogEntry, LogFilter, LogStats,
};
