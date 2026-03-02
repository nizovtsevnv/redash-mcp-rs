use serde_json::Value;
use std::sync::atomic::{AtomicU8, Ordering};

/// MCP log levels ordered by severity (lower value = more verbose).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Notice = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
    Alert = 6,
    Emergency = 7,
}

impl LogLevel {
    /// Parse a log level from its string representation.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "notice" => Some(Self::Notice),
            "warning" => Some(Self::Warning),
            "error" => Some(Self::Error),
            "critical" => Some(Self::Critical),
            "alert" => Some(Self::Alert),
            "emergency" => Some(Self::Emergency),
            _ => None,
        }
    }

    /// Return the string representation of this log level.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Notice => "notice",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
            Self::Alert => "alert",
            Self::Emergency => "emergency",
        }
    }

    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Debug,
            1 => Self::Info,
            2 => Self::Notice,
            3 => Self::Warning,
            4 => Self::Error,
            5 => Self::Critical,
            6 => Self::Alert,
            _ => Self::Emergency,
        }
    }
}

/// Thread-safe MCP log level using atomic operations for lock-free access.
pub struct McpLogLevel {
    level: AtomicU8,
}

impl McpLogLevel {
    /// Create a new log level controller with the given initial level.
    pub fn new(level: LogLevel) -> Self {
        Self {
            level: AtomicU8::new(level as u8),
        }
    }

    /// Get the current log level.
    pub fn get(&self) -> LogLevel {
        LogLevel::from_u8(self.level.load(Ordering::Relaxed))
    }

    /// Set the log level.
    pub fn set(&self, level: LogLevel) {
        self.level.store(level as u8, Ordering::Relaxed);
    }

    /// Check whether a message at the given level should be logged.
    pub fn should_log(&self, message_level: LogLevel) -> bool {
        message_level >= self.get()
    }
}

impl Default for McpLogLevel {
    fn default() -> Self {
        Self::new(LogLevel::Warning)
    }
}

/// Format a JSON-RPC log notification (server → client).
pub fn log_notification(level: LogLevel, logger: &str, data: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/message",
        "params": {
            "level": level.as_str(),
            "logger": logger,
            "data": data
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_levels() {
        assert_eq!(LogLevel::parse("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::parse("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::parse("notice"), Some(LogLevel::Notice));
        assert_eq!(LogLevel::parse("warning"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::parse("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::parse("critical"), Some(LogLevel::Critical));
        assert_eq!(LogLevel::parse("alert"), Some(LogLevel::Alert));
        assert_eq!(LogLevel::parse("emergency"), Some(LogLevel::Emergency));
        assert_eq!(LogLevel::parse("unknown"), None);
    }

    #[test]
    fn as_str_roundtrip() {
        let levels = [
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Notice,
            LogLevel::Warning,
            LogLevel::Error,
            LogLevel::Critical,
            LogLevel::Alert,
            LogLevel::Emergency,
        ];
        for level in levels {
            assert_eq!(LogLevel::parse(level.as_str()), Some(level));
        }
    }

    #[test]
    fn log_level_set_and_get() {
        let mcp = McpLogLevel::default();
        assert_eq!(mcp.get(), LogLevel::Warning);

        mcp.set(LogLevel::Debug);
        assert_eq!(mcp.get(), LogLevel::Debug);

        mcp.set(LogLevel::Emergency);
        assert_eq!(mcp.get(), LogLevel::Emergency);
    }

    #[test]
    fn should_log_filters_correctly() {
        let mcp = McpLogLevel::new(LogLevel::Warning);

        assert!(!mcp.should_log(LogLevel::Debug));
        assert!(!mcp.should_log(LogLevel::Info));
        assert!(!mcp.should_log(LogLevel::Notice));
        assert!(mcp.should_log(LogLevel::Warning));
        assert!(mcp.should_log(LogLevel::Error));
        assert!(mcp.should_log(LogLevel::Emergency));
    }

    #[test]
    fn log_notification_format() {
        let notif = log_notification(LogLevel::Error, "test-logger", "something went wrong");
        assert_eq!(notif["jsonrpc"], "2.0");
        assert_eq!(notif["method"], "notifications/message");
        assert_eq!(notif["params"]["level"], "error");
        assert_eq!(notif["params"]["logger"], "test-logger");
        assert_eq!(notif["params"]["data"], "something went wrong");
    }

    #[test]
    fn default_level_is_warning() {
        let mcp = McpLogLevel::default();
        assert_eq!(mcp.get(), LogLevel::Warning);
    }
}
