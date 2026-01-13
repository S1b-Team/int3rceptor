use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use time::OffsetDateTime;

/// Audit event severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    Info,
    Warning,
    Critical,
}

/// Audit event categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditCategory {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    Configuration,
    SystemChange,
    SecurityEvent,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp in RFC3339 format
    pub timestamp: String,
    /// Event severity
    pub severity: AuditSeverity,
    /// Event category
    pub category: AuditCategory,
    /// Action performed
    pub action: String,
    /// User or client identifier
    pub actor: String,
    /// IP address of the actor
    pub ip_address: Option<String>,
    /// Resource affected
    pub resource: Option<String>,
    /// Additional details
    pub details: Option<serde_json::Value>,
    /// Whether the action was successful
    pub success: bool,
    /// Error message if action failed
    pub error: Option<String>,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(
        severity: AuditSeverity,
        category: AuditCategory,
        action: impl Into<String>,
        actor: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| OffsetDateTime::now_utc().to_string()),
            severity,
            category,
            action: action.into(),
            actor: actor.into(),
            ip_address: None,
            resource: None,
            details: None,
            success: true,
            error: None,
        }
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip.to_string());
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Set details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Mark as failed with error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error = Some(error.into());
        self
    }
}

/// Audit logger
pub struct AuditLogger {
    log_file: Mutex<Option<File>>,
    log_path: PathBuf,
    console_output: bool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(log_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let log_path = log_path.as_ref().to_path_buf();

        // Create parent directories if they don't exist
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open log file in append mode
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        Ok(Self {
            log_file: Mutex::new(Some(file)),
            log_path,
            console_output: std::env::var("AUDIT_LOG_CONSOLE")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
        })
    }

    /// Log an audit entry
    pub fn log(&self, entry: AuditEntry) {
        // Serialize to JSON
        let json = match serde_json::to_string(&entry) {
            Ok(j) => j,
            Err(e) => {
                tracing::error!(%e, "Failed to serialize audit entry");
                return;
            }
        };

        // Write to file
        if let Ok(mut file_opt) = self.log_file.lock() {
            if let Some(file) = file_opt.as_mut() {
                if let Err(e) = writeln!(file, "{}", json) {
                    tracing::error!(%e, "Failed to write audit log entry");
                }
            }
        }

        // Log to console if enabled
        if self.console_output {
            println!("AUDIT: {}", json);
        }

        // Also log via tracing based on severity
        match entry.severity {
            AuditSeverity::Info => tracing::info!(
                category = ?entry.category,
                action = %entry.action,
                actor = %entry.actor,
                success = %entry.success,
                "Audit event"
            ),
            AuditSeverity::Warning => tracing::warn!(
                category = ?entry.category,
                action = %entry.action,
                actor = %entry.actor,
                success = %entry.success,
                "Audit event"
            ),
            AuditSeverity::Critical => tracing::error!(
                category = ?entry.category,
                action = %entry.action,
                actor = %entry.actor,
                success = %entry.success,
                "Audit event"
            ),
        }
    }

    /// Rotate log file (close current and create new)
    pub fn rotate(&self) -> std::io::Result<()> {
        let mut file_opt = self.log_file.lock().unwrap();

        // Close current file
        *file_opt = None;

        // Rename current log file with timestamp
        let timestamp = OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".to_string())
            .replace(':', "-");

        let archived_path = self.log_path.with_extension(format!("{}.log", timestamp));
        std::fs::rename(&self.log_path, archived_path)?;

        // Open new file
        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        *file_opt = Some(new_file);

        Ok(())
    }

    /// Get the log file path
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }
}

/// Audit event builder for convenience
pub struct AuditEventBuilder {
    entry: AuditEntry,
}

impl AuditEventBuilder {
    pub fn new(
        severity: AuditSeverity,
        category: AuditCategory,
        action: impl Into<String>,
        actor: impl Into<String>,
    ) -> Self {
        Self {
            entry: AuditEntry::new(severity, category, action, actor),
        }
    }

    pub fn ip(mut self, ip: IpAddr) -> Self {
        self.entry = self.entry.with_ip(ip);
        self
    }

    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.entry = self.entry.with_resource(resource);
        self
    }

    pub fn details(mut self, details: serde_json::Value) -> Self {
        self.entry = self.entry.with_details(details);
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.entry = self.entry.with_error(error);
        self
    }

    pub fn build(self) -> AuditEntry {
        self.entry
    }

    pub fn log(self, logger: &AuditLogger) {
        logger.log(self.build());
    }
}

/// Helper macro for creating audit events
#[macro_export]
macro_rules! audit_event {
    ($logger:expr, $severity:expr, $category:expr, $action:expr, $actor:expr) => {
        $crate::audit::AuditEventBuilder::new($severity, $category, $action, $actor)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            AuditSeverity::Info,
            AuditCategory::DataAccess,
            "read_data",
            "user123",
        )
        .with_ip("192.168.1.1".parse().unwrap())
        .with_resource("/api/requests")
        .with_details(serde_json::json!({"count": 10}));

        assert_eq!(entry.severity, AuditSeverity::Info);
        assert_eq!(entry.category, AuditCategory::DataAccess);
        assert_eq!(entry.action, "read_data");
        assert_eq!(entry.actor, "user123");
        assert_eq!(entry.ip_address, Some("192.168.1.1".to_string()));
        assert!(entry.success);
    }

    #[test]
    fn test_audit_logger() -> std::io::Result<()> {
        let dir = tempdir()?;
        let log_path = dir.path().join("audit.log");
        let logger = AuditLogger::new(&log_path)?;

        let entry = AuditEntry::new(
            AuditSeverity::Warning,
            AuditCategory::Authentication,
            "login_failed",
            "user456",
        );

        logger.log(entry);

        // Check file exists and contains data
        let content = fs::read_to_string(&log_path)?;
        assert!(content.contains("login_failed"));
        assert!(content.contains("user456"));

        Ok(())
    }
}
