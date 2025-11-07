//! Logging operations for debugging
//!
//! This module contains logging-related debugging operations.

use crate::error::{CodexError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: super::node::LogLevel,
    /// Log message
    pub message: String,
    /// Module or component that generated the log
    pub module: Option<String>,
    /// Additional context
    pub context: Option<serde_json::Value>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(
        level: super::node::LogLevel,
        message: String,
        module: Option<String>,
        context: Option<serde_json::Value>,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level,
            message,
            module,
            context,
        }
    }

    /// Get the log entry as a formatted string
    pub fn to_string(&self) -> String {
        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let level = self.level.to_string().to_uppercase();

        match (&self.module, &self.context) {
            (Some(module), Some(context)) => {
                format!(
                    "[{}] [{}] [{}] {} | {}",
                    timestamp, level, module, self.message, context
                )
            }
            (Some(module), None) => {
                format!("[{}] [{}] [{}] {}", timestamp, level, module, self.message)
            }
            (None, Some(context)) => {
                format!("[{}] [{}] {} | {}", timestamp, level, self.message, context)
            }
            (None, None) => {
                format!("[{}] [{}] {}", timestamp, level, self.message)
            }
        }
    }

    /// Get the log entry as JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| CodexError::library_error(format!("Failed to serialize log entry: {}", e)))
    }
}

/// Log filter criteria
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    /// Minimum log level to include
    pub min_level: Option<super::node::LogLevel>,
    /// Maximum log level to include
    pub max_level: Option<super::node::LogLevel>,
    /// Module filter (include only these modules)
    pub modules: Vec<String>,
    /// Message pattern to search for
    pub message_pattern: Option<String>,
    /// Start time for filtering
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// End time for filtering
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Limit the number of results
    pub limit: Option<usize>,
}

impl LogFilter {
    /// Create a new log filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum log level
    pub fn min_level(mut self, level: super::node::LogLevel) -> Self {
        self.min_level = Some(level);
        self
    }

    /// Set the maximum log level
    pub fn max_level(mut self, level: super::node::LogLevel) -> Self {
        self.max_level = Some(level);
        self
    }

    /// Add a module to filter by
    pub fn module(mut self, module: String) -> Self {
        self.modules.push(module);
        self
    }

    /// Set the message pattern
    pub fn message_pattern(mut self, pattern: String) -> Self {
        self.message_pattern = Some(pattern);
        self
    }

    /// Set the start time
    pub fn start_time(mut self, time: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_time = Some(time);
        self
    }

    /// Set the end time
    pub fn end_time(mut self, time: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_time = Some(time);
        self
    }

    /// Set the result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if a log entry matches this filter
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Check log level range
        if let Some(min_level) = &self.min_level {
            if !self.level_ge(entry.level, *min_level) {
                return false;
            }
        }

        if let Some(max_level) = &self.max_level {
            if !self.level_le(entry.level, *max_level) {
                return false;
            }
        }

        // Check module filter
        if !self.modules.is_empty() {
            if let Some(module) = &entry.module {
                if !self.modules.contains(module) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check message pattern
        if let Some(pattern) = &self.message_pattern {
            if !entry
                .message
                .to_lowercase()
                .contains(&pattern.to_lowercase())
            {
                return false;
            }
        }

        // Check time range
        if let Some(start_time) = &self.start_time {
            if entry.timestamp < *start_time {
                return false;
            }
        }

        if let Some(end_time) = &self.end_time {
            if entry.timestamp > *end_time {
                return false;
            }
        }

        true
    }

    fn level_ge(&self, a: super::node::LogLevel, b: super::node::LogLevel) -> bool {
        use super::node::LogLevel;
        match (a, b) {
            (LogLevel::Trace, LogLevel::Trace) => true,
            (LogLevel::Trace, _) => true,
            (LogLevel::Debug, LogLevel::Trace) => false,
            (LogLevel::Debug, LogLevel::Debug) => true,
            (LogLevel::Debug, _) => true,
            (LogLevel::Info, LogLevel::Trace | LogLevel::Debug) => false,
            (LogLevel::Info, LogLevel::Info) => true,
            (LogLevel::Info, _) => true,
            (LogLevel::Notice, LogLevel::Trace | LogLevel::Debug | LogLevel::Info) => false,
            (LogLevel::Notice, LogLevel::Notice) => true,
            (LogLevel::Notice, _) => true,
            (
                LogLevel::Warn,
                LogLevel::Trace | LogLevel::Debug | LogLevel::Info | LogLevel::Notice,
            ) => false,
            (LogLevel::Warn, LogLevel::Warn) => true,
            (LogLevel::Warn, _) => true,
            (LogLevel::Error, LogLevel::Fatal) => false,
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Error, _) => true,
            (LogLevel::Fatal, LogLevel::Fatal) => true,
            (LogLevel::Fatal, _) => false,
        }
    }

    fn level_le(&self, a: super::node::LogLevel, b: super::node::LogLevel) -> bool {
        self.level_ge(b, a)
    }
}

/// Log statistics
#[derive(Debug, Clone, Default)]
pub struct LogStats {
    /// Count of logs by level
    pub counts_by_level: HashMap<super::node::LogLevel, usize>,
    /// Count of logs by module
    pub counts_by_module: HashMap<String, usize>,
    /// Total number of log entries
    pub total_entries: usize,
    /// Time range of logs
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
}

impl LogStats {
    /// Create new log stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a log entry to the statistics
    pub fn add_entry(&mut self, entry: &LogEntry) {
        *self.counts_by_level.entry(entry.level).or_insert(0) += 1;
        self.total_entries += 1;

        if let Some(module) = &entry.module {
            *self.counts_by_module.entry(module.clone()).or_insert(0) += 1;
        }

        match &mut self.time_range {
            Some((start, end)) => {
                if entry.timestamp < *start {
                    *start = entry.timestamp;
                }
                if entry.timestamp > *end {
                    *end = entry.timestamp;
                }
            }
            None => {
                self.time_range = Some((entry.timestamp, entry.timestamp));
            }
        }
    }

    /// Get the most common log level
    pub fn most_common_level(&self) -> Option<super::node::LogLevel> {
        self.counts_by_level
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(level, _)| *level)
    }

    /// Get the most active module
    pub fn most_active_module(&self) -> Option<String> {
        self.counts_by_module
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(module, _)| module.clone())
    }

    /// Get the percentage of logs at or above the given level
    pub fn percentage_at_or_above(&self, level: super::node::LogLevel) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            let mut count = 0;
            for (log_level, &entry_count) in &self.counts_by_level {
                if LogFilter::new()
                    .min_level(*log_level)
                    .matches(&LogEntry::new(*log_level, "test".to_string(), None, None))
                {
                    count += entry_count;
                }
            }
            count as f64 / self.total_entries as f64 * 100.0
        }
    }
}

/// Read log entries from a file
///
/// # Arguments
///
/// * `file_path` - Path to the log file
/// * `filter` - Optional filter to apply
///
/// # Returns
///
/// Vector of log entries
pub fn read_log_file<P: AsRef<Path>>(
    file_path: P,
    filter: Option<LogFilter>,
) -> Result<Vec<LogEntry>> {
    let content = fs::read_to_string(file_path).map_err(|e| CodexError::Io(e))?;

    let mut entries = Vec::new();
    let filter = filter.unwrap_or_default();

    for line in content.lines() {
        if let Ok(entry) = parse_log_line(line) {
            if filter.matches(&entry) {
                entries.push(entry);
            }
        }
    }

    // Apply limit if specified
    if let Some(limit) = filter.limit {
        entries.truncate(limit);
    }

    Ok(entries)
}

/// Parse a single log line into a LogEntry
///
/// # Arguments
///
/// * `line` - The log line to parse
///
/// # Returns
///
/// Parsed log entry or error
pub fn parse_log_line(line: &str) -> Result<LogEntry> {
    // Expected format: [timestamp] [level] [module] message | context
    // or: [timestamp] [level] message | context
    // or: [timestamp] [level] message

    let line = line.trim();
    if line.is_empty() {
        return Err(CodexError::invalid_parameter("line", "Empty log line"));
    }

    // Extract timestamp
    if !line.starts_with('[') {
        return Err(CodexError::invalid_parameter(
            "line",
            "Invalid log format - missing timestamp",
        ));
    }

    let timestamp_end = line.find(']').ok_or_else(|| {
        CodexError::invalid_parameter("line", "Invalid log format - unclosed timestamp")
    })?;
    let timestamp_str = &line[1..timestamp_end];

    let timestamp = chrono::DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%.3f")
        .map_err(|_| CodexError::invalid_parameter("line", "Invalid timestamp format"))?
        .with_timezone(&chrono::Utc);

    let remaining = &line[timestamp_end + 1..].trim();

    // Extract level
    if !remaining.starts_with('[') {
        return Err(CodexError::invalid_parameter(
            "line",
            "Invalid log format - missing level",
        ));
    }

    let level_end = remaining.find(']').ok_or_else(|| {
        CodexError::invalid_parameter("line", "Invalid log format - unclosed level")
    })?;
    let level_str = &remaining[1..level_end];

    let level = match level_str.to_lowercase().as_str() {
        "trace" => super::node::LogLevel::Trace,
        "debug" => super::node::LogLevel::Debug,
        "info" => super::node::LogLevel::Info,
        "notice" => super::node::LogLevel::Notice,
        "warn" => super::node::LogLevel::Warn,
        "error" => super::node::LogLevel::Error,
        "fatal" => super::node::LogLevel::Fatal,
        _ => {
            return Err(CodexError::invalid_parameter("line", "Invalid log level"));
        }
    };

    let remaining = &remaining[level_end + 1..].trim();

    // Check for module
    let (module, message_and_context) = if remaining.starts_with('[') {
        let module_end = remaining.find(']').ok_or_else(|| {
            CodexError::invalid_parameter("line", "Invalid log format - unclosed module")
        })?;
        let module_str = &remaining[1..module_end];
        let remaining_trimmed = remaining[module_end + 1..].trim();
        (Some(module_str.to_string()), remaining_trimmed)
    } else {
        (None, *remaining)
    };

    // Split message and context
    let (message, context) = if let Some(sep_pos) = message_and_context.find(" | ") {
        let message = &message_and_context[..sep_pos];
        let context_str = &message_and_context[sep_pos + 3..];

        let context: serde_json::Value = serde_json::from_str(context_str)
            .map_err(|_| CodexError::invalid_parameter("line", "Invalid context JSON"))?;

        (message.to_string(), Some(context))
    } else {
        (message_and_context.to_string(), None)
    };

    Ok(LogEntry {
        timestamp,
        level,
        message,
        module,
        context,
    })
}

/// Analyze log patterns
///
/// # Arguments
///
/// * `entries` - Log entries to analyze
///
/// # Returns
///
/// Log analysis results
pub fn analyze_log_patterns(entries: &[LogEntry]) -> LogAnalysis {
    let mut stats = LogStats::new();
    let mut error_patterns = HashMap::new();
    let mut warning_patterns = HashMap::new();

    for entry in entries {
        stats.add_entry(entry);

        // Analyze error patterns
        if matches!(
            entry.level,
            super::node::LogLevel::Error | super::node::LogLevel::Fatal
        ) {
            let pattern = extract_error_pattern(&entry.message);
            *error_patterns.entry(pattern).or_insert(0) += 1;
        }

        // Analyze warning patterns
        if matches!(entry.level, super::node::LogLevel::Warn) {
            let pattern = extract_warning_pattern(&entry.message);
            *warning_patterns.entry(pattern).or_insert(0) += 1;
        }
    }

    LogAnalysis {
        stats: stats.clone(),
        error_patterns: error_patterns.clone(),
        warning_patterns: warning_patterns.clone(),
        recommendations: generate_recommendations(&stats, &error_patterns, &warning_patterns),
    }
}

/// Log analysis results
#[derive(Debug, Clone)]
pub struct LogAnalysis {
    /// Log statistics
    pub stats: LogStats,
    /// Common error patterns
    pub error_patterns: HashMap<String, usize>,
    /// Common warning patterns
    pub warning_patterns: HashMap<String, usize>,
    /// Recommendations based on analysis
    pub recommendations: Vec<String>,
}

fn extract_error_pattern(message: &str) -> String {
    // Simple pattern extraction - in a real implementation, you might use
    // more sophisticated pattern matching or machine learning
    let message = message.to_lowercase();

    if message.contains("connection") {
        "connection_error".to_string()
    } else if message.contains("timeout") {
        "timeout_error".to_string()
    } else if message.contains("permission") || message.contains("access") {
        "permission_error".to_string()
    } else if message.contains("file") || message.contains("disk") {
        "file_error".to_string()
    } else if message.contains("memory") || message.contains("oom") {
        "memory_error".to_string()
    } else {
        "other_error".to_string()
    }
}

fn extract_warning_pattern(message: &str) -> String {
    let message = message.to_lowercase();

    if message.contains("retry") {
        "retry_warning".to_string()
    } else if message.contains("slow") || message.contains("latency") {
        "performance_warning".to_string()
    } else if message.contains("deprecated") {
        "deprecation_warning".to_string()
    } else if message.contains("quota") || message.contains("limit") {
        "quota_warning".to_string()
    } else {
        "other_warning".to_string()
    }
}

fn generate_recommendations(
    stats: &LogStats,
    error_patterns: &HashMap<String, usize>,
    warning_patterns: &HashMap<String, usize>,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Check error rate
    let error_rate = stats.percentage_at_or_above(super::node::LogLevel::Error);
    if error_rate > 5.0 {
        recommendations.push(
            "High error rate detected (>5%). Consider investigating system health.".to_string(),
        );
    }

    // Check for common error patterns
    if let Some((pattern, count)) = error_patterns.iter().max_by_key(|(_, &count)| count) {
        if *count > 10 {
            recommendations.push(format!(
                "Frequent '{}' errors detected ({} occurrences). Consider addressing this issue.",
                pattern, count
            ));
        }
    }

    // Check for performance warnings
    if warning_patterns.contains_key("performance_warning") {
        recommendations.push(
            "Performance warnings detected. Consider optimizing system resources.".to_string(),
        );
    }

    // Check for quota warnings
    if warning_patterns.contains_key("quota_warning") {
        recommendations.push(
            "Quota warnings detected. Consider increasing storage or network limits.".to_string(),
        );
    }

    // Check log volume
    if stats.total_entries > 10000 {
        recommendations.push(
            "High log volume detected. Consider adjusting log levels or implementing log rotation."
                .to_string(),
        );
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::super::node::LogLevel;
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(
            LogLevel::Info,
            "Test message".to_string(),
            Some("test_module".to_string()),
            Some(serde_json::json!({"key": "value"})),
        );

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.module, Some("test_module".to_string()));
        assert!(entry.context.is_some());
    }

    #[test]
    fn test_log_entry_to_string() {
        let entry = LogEntry::new(
            LogLevel::Info,
            "Test message".to_string(),
            Some("test_module".to_string()),
            Some(serde_json::json!({"key": "value"})),
        );

        let string = entry.to_string();
        assert!(string.contains("INFO"));
        assert!(string.contains("test_module"));
        assert!(string.contains("Test message"));
        assert!(string.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_log_filter() {
        let filter = LogFilter::new()
            .min_level(LogLevel::Warn)
            .module("test_module".to_string())
            .message_pattern("error".to_string())
            .limit(10);

        assert_eq!(filter.min_level, Some(LogLevel::Warn));
        assert_eq!(filter.modules, vec!["test_module"]);
        assert_eq!(filter.message_pattern, Some("error".to_string()));
        assert_eq!(filter.limit, Some(10));
    }

    #[test]
    fn test_log_filter_matches() {
        let filter = LogFilter::new().min_level(LogLevel::Warn);

        let warn_entry = LogEntry::new(LogLevel::Warn, "Warning".to_string(), None, None);
        let info_entry = LogEntry::new(LogLevel::Info, "Info".to_string(), None, None);

        assert!(filter.matches(&warn_entry));
        assert!(!filter.matches(&info_entry));
    }

    #[test]
    fn test_log_stats() {
        let mut stats = LogStats::new();

        let entry1 = LogEntry::new(
            LogLevel::Info,
            "Test".to_string(),
            Some("module1".to_string()),
            None,
        );
        let entry2 = LogEntry::new(
            LogLevel::Error,
            "Error".to_string(),
            Some("module1".to_string()),
            None,
        );
        let entry3 = LogEntry::new(
            LogLevel::Warn,
            "Warning".to_string(),
            Some("module2".to_string()),
            None,
        );

        stats.add_entry(&entry1);
        stats.add_entry(&entry2);
        stats.add_entry(&entry3);

        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.counts_by_level.get(&LogLevel::Info), Some(&1));
        assert_eq!(stats.counts_by_level.get(&LogLevel::Error), Some(&1));
        assert_eq!(stats.counts_by_level.get(&LogLevel::Warn), Some(&1));
        assert_eq!(stats.counts_by_module.get("module1"), Some(&2));
        assert_eq!(stats.counts_by_module.get("module2"), Some(&1));
    }

    #[test]
    fn test_parse_log_line() {
        let line =
            "[2023-01-01 12:00:00.000] [INFO] [test_module] Test message | {\"key\": \"value\"}";
        let entry = parse_log_line(line).unwrap();

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.module, Some("test_module".to_string()));
        assert_eq!(entry.message, "Test message");
        assert!(entry.context.is_some());
    }

    #[test]
    fn test_parse_log_line_no_module() {
        let line = "[2023-01-01 12:00:00.000] [INFO] Test message";
        let entry = parse_log_line(line).unwrap();

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.module, None);
        assert_eq!(entry.message, "Test message");
        assert!(entry.context.is_none());
    }

    #[test]
    fn test_parse_log_line_invalid() {
        let line = "invalid log line";
        let result = parse_log_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_log_patterns() {
        let entries = vec![
            LogEntry::new(LogLevel::Error, "Connection failed".to_string(), None, None),
            LogEntry::new(
                LogLevel::Error,
                "Connection timeout".to_string(),
                None,
                None,
            ),
            LogEntry::new(LogLevel::Warn, "Slow response".to_string(), None, None),
            LogEntry::new(LogLevel::Info, "Normal operation".to_string(), None, None),
        ];

        let analysis = analyze_log_patterns(&entries);

        assert_eq!(analysis.stats.total_entries, 4);
        assert_eq!(analysis.error_patterns.get("connection_error"), Some(&2));
        assert_eq!(
            analysis.warning_patterns.get("performance_warning"),
            Some(&1)
        );
        assert!(!analysis.recommendations.is_empty());
    }

    #[test]
    fn test_extract_error_pattern() {
        assert_eq!(
            extract_error_pattern("Connection failed"),
            "connection_error"
        );
        assert_eq!(extract_error_pattern("Request timeout"), "timeout_error");
        assert_eq!(
            extract_error_pattern("Permission denied"),
            "permission_error"
        );
        assert_eq!(extract_error_pattern("File not found"), "file_error");
        assert_eq!(extract_error_pattern("Out of memory"), "memory_error");
        assert_eq!(extract_error_pattern("Unknown error"), "other_error");
    }

    #[test]
    fn test_extract_warning_pattern() {
        assert_eq!(
            extract_warning_pattern("Retrying operation"),
            "retry_warning"
        );
        assert_eq!(
            extract_warning_pattern("Slow query detected"),
            "performance_warning"
        );
        assert_eq!(
            extract_warning_pattern("Deprecated API used"),
            "deprecation_warning"
        );
        assert_eq!(extract_warning_pattern("Quota exceeded"), "quota_warning");
        assert_eq!(extract_warning_pattern("General warning"), "other_warning");
    }
}
