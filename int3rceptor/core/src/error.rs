//! Error types for INT3RCEPTOR Core
//!
//! This module provides a unified error handling system with:
//! - Specific error variants for each subsystem
//! - Error codes for API responses
//! - Context-aware error creation
//! - Safe lock access helpers

use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use thiserror::Error;

/// Result type alias for INT3RCEPTOR operations
pub type Result<T> = std::result::Result<T, ProxyError>;

/// Error codes for API responses and logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ErrorCode {
    // General errors (1000-1099)
    Unknown = 1000,
    Internal = 1001,
    InvalidConfig = 1002,
    NotFound = 1003,
    AlreadyExists = 1004,

    // IO/Network errors (1100-1199)
    IoError = 1100,
    ConnectionFailed = 1101,
    ConnectionTimeout = 1102,
    ConnectionClosed = 1103,
    DnsResolution = 1104,

    // HTTP errors (1200-1299)
    HttpParse = 1200,
    HttpInvalidRequest = 1201,
    HttpInvalidResponse = 1202,
    HttpInvalidUri = 1203,
    HttpInvalidHeader = 1204,
    HttpBodyTooLarge = 1205,

    // TLS errors (1300-1399)
    TlsHandshake = 1300,
    TlsCertificate = 1301,
    TlsVerification = 1302,
    TlsProtocol = 1303,

    // Proxy errors (1400-1499)
    ProxyNotRunning = 1400,
    ProxyAlreadyRunning = 1401,
    ProxyUpstreamFailed = 1402,
    ProxyConnectFailed = 1403,

    // Storage errors (1500-1599)
    DatabaseConnection = 1500,
    DatabaseQuery = 1501,
    DatabaseMigration = 1502,
    StorageFull = 1503,

    // Rules errors (1600-1699)
    RuleInvalid = 1600,
    RuleNotFound = 1601,
    RuleRegexInvalid = 1602,
    RuleConflict = 1603,

    // Scope errors (1700-1799)
    ScopeInvalid = 1700,
    ScopePatternInvalid = 1701,

    // Intruder errors (1800-1899)
    IntruderJobNotFound = 1800,
    IntruderInvalidPayload = 1801,
    IntruderRateLimited = 1802,

    // WebSocket errors (1900-1999)
    WebSocketParse = 1900,
    WebSocketClosed = 1901,
    WebSocketProtocol = 1902,

    // License errors (2000-2099)
    LicenseInvalid = 2000,
    LicenseExpired = 2001,
    LicenseFeatureDisabled = 2002,

    // Lock errors (2100-2199)
    LockPoisoned = 2100,
    LockTimeout = 2101,
}

impl ErrorCode {
    /// Get the numeric code
    pub fn code(&self) -> u16 {
        *self as u16
    }

    /// Create ErrorCode from numeric code
    pub fn from_code(code: u16) -> Self {
        match code {
            1000 => Self::Unknown,
            1001 => Self::Internal,
            1002 => Self::InvalidConfig,
            1003 => Self::NotFound,
            1004 => Self::AlreadyExists,
            1100 => Self::IoError,
            1101 => Self::ConnectionFailed,
            1102 => Self::ConnectionTimeout,
            1103 => Self::ConnectionClosed,
            1104 => Self::DnsResolution,
            1200 => Self::HttpParse,
            1201 => Self::HttpInvalidRequest,
            1202 => Self::HttpInvalidResponse,
            1203 => Self::HttpInvalidUri,
            1204 => Self::HttpInvalidHeader,
            1205 => Self::HttpBodyTooLarge,
            1300 => Self::TlsHandshake,
            1301 => Self::TlsCertificate,
            1302 => Self::TlsVerification,
            1303 => Self::TlsProtocol,
            1400 => Self::ProxyNotRunning,
            1401 => Self::ProxyAlreadyRunning,
            1402 => Self::ProxyUpstreamFailed,
            1403 => Self::ProxyConnectFailed,
            1500 => Self::DatabaseConnection,
            1501 => Self::DatabaseQuery,
            1502 => Self::DatabaseMigration,
            1503 => Self::StorageFull,
            1600 => Self::RuleInvalid,
            1601 => Self::RuleNotFound,
            1602 => Self::RuleRegexInvalid,
            1603 => Self::RuleConflict,
            1700 => Self::ScopeInvalid,
            1701 => Self::ScopePatternInvalid,
            1800 => Self::IntruderJobNotFound,
            1801 => Self::IntruderInvalidPayload,
            1802 => Self::IntruderRateLimited,
            1900 => Self::WebSocketParse,
            1901 => Self::WebSocketClosed,
            1902 => Self::WebSocketProtocol,
            2000 => Self::LicenseInvalid,
            2001 => Self::LicenseExpired,
            2002 => Self::LicenseFeatureDisabled,
            2100 => Self::LockPoisoned,
            2101 => Self::LockTimeout,
            _ => Self::Unknown,
        }
    }

    /// Get a short string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN",
            Self::Internal => "INTERNAL",
            Self::InvalidConfig => "INVALID_CONFIG",
            Self::NotFound => "NOT_FOUND",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::IoError => "IO_ERROR",
            Self::ConnectionFailed => "CONNECTION_FAILED",
            Self::ConnectionTimeout => "CONNECTION_TIMEOUT",
            Self::ConnectionClosed => "CONNECTION_CLOSED",
            Self::DnsResolution => "DNS_RESOLUTION",
            Self::HttpParse => "HTTP_PARSE",
            Self::HttpInvalidRequest => "HTTP_INVALID_REQUEST",
            Self::HttpInvalidResponse => "HTTP_INVALID_RESPONSE",
            Self::HttpInvalidUri => "HTTP_INVALID_URI",
            Self::HttpInvalidHeader => "HTTP_INVALID_HEADER",
            Self::HttpBodyTooLarge => "HTTP_BODY_TOO_LARGE",
            Self::TlsHandshake => "TLS_HANDSHAKE",
            Self::TlsCertificate => "TLS_CERTIFICATE",
            Self::TlsVerification => "TLS_VERIFICATION",
            Self::TlsProtocol => "TLS_PROTOCOL",
            Self::ProxyNotRunning => "PROXY_NOT_RUNNING",
            Self::ProxyAlreadyRunning => "PROXY_ALREADY_RUNNING",
            Self::ProxyUpstreamFailed => "PROXY_UPSTREAM_FAILED",
            Self::ProxyConnectFailed => "PROXY_CONNECT_FAILED",
            Self::DatabaseConnection => "DATABASE_CONNECTION",
            Self::DatabaseQuery => "DATABASE_QUERY",
            Self::DatabaseMigration => "DATABASE_MIGRATION",
            Self::StorageFull => "STORAGE_FULL",
            Self::RuleInvalid => "RULE_INVALID",
            Self::RuleNotFound => "RULE_NOT_FOUND",
            Self::RuleRegexInvalid => "RULE_REGEX_INVALID",
            Self::RuleConflict => "RULE_CONFLICT",
            Self::ScopeInvalid => "SCOPE_INVALID",
            Self::ScopePatternInvalid => "SCOPE_PATTERN_INVALID",
            Self::IntruderJobNotFound => "INTRUDER_JOB_NOT_FOUND",
            Self::IntruderInvalidPayload => "INTRUDER_INVALID_PAYLOAD",
            Self::IntruderRateLimited => "INTRUDER_RATE_LIMITED",
            Self::WebSocketParse => "WEBSOCKET_PARSE",
            Self::WebSocketClosed => "WEBSOCKET_CLOSED",
            Self::WebSocketProtocol => "WEBSOCKET_PROTOCOL",
            Self::LicenseInvalid => "LICENSE_INVALID",
            Self::LicenseExpired => "LICENSE_EXPIRED",
            Self::LicenseFeatureDisabled => "LICENSE_FEATURE_DISABLED",
            Self::LockPoisoned => "LOCK_POISONED",
            Self::LockTimeout => "LOCK_TIMEOUT",
        }
    }
}

/// Main error type for INT3RCEPTOR operations
#[derive(Debug, Error)]
pub enum ProxyError {
    // === IO/Network Errors ===
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Connection failed to {host}: {reason}")]
    ConnectionFailed { host: String, reason: String },

    #[error("Connection timeout after {timeout_ms}ms to {host}")]
    ConnectionTimeout { host: String, timeout_ms: u64 },

    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    // === HTTP Errors ===
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] hyper::http::Error),

    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] hyper::http::uri::InvalidUri),

    #[error("Invalid URI: {uri} - {reason}")]
    InvalidUriWithContext { uri: String, reason: String },

    #[error("HTTP client error: {0}")]
    Client(#[from] hyper_util::client::legacy::Error),

    #[error("Invalid HTTP request: {0}")]
    InvalidRequest(String),

    #[error("Invalid HTTP response: {0}")]
    InvalidResponse(String),

    #[error("Invalid header {name}: {reason}")]
    InvalidHeader { name: String, reason: String },

    #[error("Request body too large: {size} bytes (max: {max})")]
    BodyTooLarge { size: usize, max: usize },

    // === TLS Errors ===
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    #[error("TLS handshake failed for {host}: {reason}")]
    TlsHandshake { host: String, reason: String },

    #[error("Certificate error: {0}")]
    Certificate(#[from] rcgen::Error),

    #[error("Certificate generation failed for {domain}: {reason}")]
    CertificateGeneration { domain: String, reason: String },

    #[error("Certificate verification failed: {0}")]
    CertificateVerification(String),

    // === Proxy Errors ===
    #[error("Proxy is not running")]
    ProxyNotRunning,

    #[error("Proxy is already running on port {port}")]
    ProxyAlreadyRunning { port: u16 },

    #[error("Upstream request failed: {0}")]
    UpstreamFailed(String),

    #[error("CONNECT tunnel failed to {host}:{port}")]
    ConnectFailed { host: String, port: u16 },

    // === Storage Errors ===
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Database connection failed: {0}")]
    DatabaseConnection(String),

    #[error("Database query failed: {query} - {reason}")]
    DatabaseQuery { query: String, reason: String },

    #[error("Storage is full: {used} / {capacity} bytes")]
    StorageFull { used: u64, capacity: u64 },

    // === Serialization Errors ===
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Failed to serialize {what}: {reason}")]
    SerializationFailed { what: String, reason: String },

    #[error("Failed to deserialize {what}: {reason}")]
    DeserializationFailed { what: String, reason: String },

    // === Rules Errors ===
    #[error("Invalid rule: {0}")]
    RuleInvalid(String),

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Invalid regex pattern: {pattern} - {reason}")]
    RegexInvalid { pattern: String, reason: String },

    // === Scope Errors ===
    #[error("Invalid scope configuration: {0}")]
    ScopeInvalid(String),

    #[error("Invalid scope pattern: {pattern} - {reason}")]
    ScopePatternInvalid { pattern: String, reason: String },

    // === Intruder Errors ===
    #[error("Intruder job not found: {0}")]
    IntruderJobNotFound(String),

    #[error("Invalid payload: {0}")]
    IntruderInvalidPayload(String),

    #[error("Rate limited: retry after {retry_after_ms}ms")]
    IntruderRateLimited { retry_after_ms: u64 },

    // === WebSocket Errors ===
    #[error("WebSocket parse error: {0}")]
    WebSocketParse(String),

    #[error("WebSocket connection closed: code={code}")]
    WebSocketClosed { code: u16 },

    #[error("WebSocket protocol error: {0}")]
    WebSocketProtocol(String),

    // === License Errors ===
    #[error("Invalid license: {0}")]
    LicenseInvalid(String),

    #[error("License expired on {expiry}")]
    LicenseExpired { expiry: String },

    #[error("Feature '{feature}' requires {required_tier} license")]
    LicenseFeatureDisabled {
        feature: String,
        required_tier: String,
    },

    // === Lock Errors ===
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    #[error("Lock acquisition timeout")]
    LockTimeout,

    // === Configuration Errors ===
    #[error("Invalid configuration: {field} - {reason}")]
    InvalidConfig { field: String, reason: String },

    // === Generic Errors ===
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ProxyError {
    /// Get the error code for this error
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Io(_) => ErrorCode::IoError,
            Self::ConnectionFailed { .. } => ErrorCode::ConnectionFailed,
            Self::ConnectionTimeout { .. } => ErrorCode::ConnectionTimeout,
            Self::ConnectionClosed => ErrorCode::ConnectionClosed,
            Self::Hyper(_) | Self::Http(_) | Self::Client(_) => ErrorCode::HttpParse,
            Self::InvalidUri(_) | Self::InvalidUriWithContext { .. } => ErrorCode::HttpInvalidUri,
            Self::InvalidRequest(_) => ErrorCode::HttpInvalidRequest,
            Self::InvalidResponse(_) => ErrorCode::HttpInvalidResponse,
            Self::InvalidHeader { .. } => ErrorCode::HttpInvalidHeader,
            Self::BodyTooLarge { .. } => ErrorCode::HttpBodyTooLarge,
            Self::Tls(_) | Self::TlsHandshake { .. } => ErrorCode::TlsHandshake,
            Self::Certificate(_) | Self::CertificateGeneration { .. } => ErrorCode::TlsCertificate,
            Self::CertificateVerification(_) => ErrorCode::TlsVerification,
            Self::ProxyNotRunning => ErrorCode::ProxyNotRunning,
            Self::ProxyAlreadyRunning { .. } => ErrorCode::ProxyAlreadyRunning,
            Self::UpstreamFailed(_) => ErrorCode::ProxyUpstreamFailed,
            Self::ConnectFailed { .. } => ErrorCode::ProxyConnectFailed,
            Self::Database(_) | Self::DatabaseConnection(_) => ErrorCode::DatabaseConnection,
            Self::DatabaseQuery { .. } => ErrorCode::DatabaseQuery,
            Self::StorageFull { .. } => ErrorCode::StorageFull,
            Self::Serde(_)
            | Self::SerializationFailed { .. }
            | Self::DeserializationFailed { .. } => ErrorCode::Internal,
            Self::RuleInvalid(_) => ErrorCode::RuleInvalid,
            Self::RuleNotFound(_) => ErrorCode::RuleNotFound,
            Self::RegexInvalid { .. } => ErrorCode::RuleRegexInvalid,
            Self::ScopeInvalid(_) => ErrorCode::ScopeInvalid,
            Self::ScopePatternInvalid { .. } => ErrorCode::ScopePatternInvalid,
            Self::IntruderJobNotFound(_) => ErrorCode::IntruderJobNotFound,
            Self::IntruderInvalidPayload(_) => ErrorCode::IntruderInvalidPayload,
            Self::IntruderRateLimited { .. } => ErrorCode::IntruderRateLimited,
            Self::WebSocketParse(_) => ErrorCode::WebSocketParse,
            Self::WebSocketClosed { .. } => ErrorCode::WebSocketClosed,
            Self::WebSocketProtocol(_) => ErrorCode::WebSocketProtocol,
            Self::LicenseInvalid(_) => ErrorCode::LicenseInvalid,
            Self::LicenseExpired { .. } => ErrorCode::LicenseExpired,
            Self::LicenseFeatureDisabled { .. } => ErrorCode::LicenseFeatureDisabled,
            Self::LockPoisoned(_) => ErrorCode::LockPoisoned,
            Self::LockTimeout => ErrorCode::LockTimeout,
            Self::InvalidConfig { .. } => ErrorCode::InvalidConfig,
            Self::NotFound(_) => ErrorCode::NotFound,
            Self::AlreadyExists(_) => ErrorCode::AlreadyExists,
            Self::Internal(_) => ErrorCode::Internal,
        }
    }

    /// Check if this error is recoverable (can retry)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionTimeout { .. }
                | Self::ConnectionClosed
                | Self::LockTimeout
                | Self::IntruderRateLimited { .. }
                | Self::UpstreamFailed(_)
        )
    }

    /// Check if this error is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::InvalidRequest(_)
                | Self::InvalidUri(_)
                | Self::InvalidUriWithContext { .. }
                | Self::InvalidHeader { .. }
                | Self::BodyTooLarge { .. }
                | Self::RuleInvalid(_)
                | Self::ScopeInvalid(_)
                | Self::IntruderInvalidPayload(_)
                | Self::InvalidConfig { .. }
                | Self::NotFound(_)
        )
    }

    // === Constructor helpers ===

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a not found error
    pub fn not_found(what: impl Into<String>) -> Self {
        Self::NotFound(what.into())
    }

    /// Create a connection failed error
    pub fn connection_failed(host: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            host: host.into(),
            reason: reason.into(),
        }
    }

    /// Create a TLS handshake error
    pub fn tls_handshake(host: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::TlsHandshake {
            host: host.into(),
            reason: reason.into(),
        }
    }

    /// Create a database query error
    pub fn db_query(query: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::DatabaseQuery {
            query: query.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid config error
    pub fn invalid_config(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidConfig {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

// === From implementations for common error types ===

impl From<anyhow::Error> for ProxyError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value.to_string())
    }
}

impl From<regex::Error> for ProxyError {
    fn from(value: regex::Error) -> Self {
        Self::RegexInvalid {
            pattern: String::new(),
            reason: value.to_string(),
        }
    }
}

impl<T> From<PoisonError<T>> for ProxyError {
    fn from(value: PoisonError<T>) -> Self {
        Self::LockPoisoned(value.to_string())
    }
}

// === Lock helper extension traits ===

/// Extension trait for safe RwLock access
pub trait RwLockExt<T> {
    /// Try to acquire read lock, returning ProxyError on failure
    fn try_read_safe(&self) -> Result<RwLockReadGuard<'_, T>>;

    /// Try to acquire write lock, returning ProxyError on failure
    fn try_write_safe(&self) -> Result<RwLockWriteGuard<'_, T>>;

    /// Acquire read lock, converting poison error to ProxyError
    fn read_safe(&self) -> Result<RwLockReadGuard<'_, T>>;

    /// Acquire write lock, converting poison error to ProxyError
    fn write_safe(&self) -> Result<RwLockWriteGuard<'_, T>>;
}

impl<T> RwLockExt<T> for RwLock<T> {
    fn try_read_safe(&self) -> Result<RwLockReadGuard<'_, T>> {
        self.try_read().map_err(|_| ProxyError::LockTimeout)
    }

    fn try_write_safe(&self) -> Result<RwLockWriteGuard<'_, T>> {
        self.try_write().map_err(|_| ProxyError::LockTimeout)
    }

    fn read_safe(&self) -> Result<RwLockReadGuard<'_, T>> {
        self.read()
            .map_err(|e| ProxyError::LockPoisoned(e.to_string()))
    }

    fn write_safe(&self) -> Result<RwLockWriteGuard<'_, T>> {
        self.write()
            .map_err(|e| ProxyError::LockPoisoned(e.to_string()))
    }
}

/// Extension trait for parking_lot RwLock (non-poisoning)
pub trait ParkingLotRwLockExt<T> {
    /// Acquire read lock (parking_lot never poisons)
    fn read_safe(&self) -> parking_lot::RwLockReadGuard<'_, T>;

    /// Acquire write lock (parking_lot never poisons)
    fn write_safe(&self) -> parking_lot::RwLockWriteGuard<'_, T>;
}

impl<T> ParkingLotRwLockExt<T> for parking_lot::RwLock<T> {
    fn read_safe(&self) -> parking_lot::RwLockReadGuard<'_, T> {
        self.read()
    }

    fn write_safe(&self) -> parking_lot::RwLockWriteGuard<'_, T> {
        self.write()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = ProxyError::ProxyNotRunning;
        assert_eq!(err.code(), ErrorCode::ProxyNotRunning);
        assert_eq!(err.code().code(), 1400);
        assert_eq!(err.code().as_str(), "PROXY_NOT_RUNNING");
    }

    #[test]
    fn test_error_recoverable() {
        assert!(ProxyError::ConnectionTimeout {
            host: "test".into(),
            timeout_ms: 1000
        }
        .is_recoverable());
        assert!(!ProxyError::InvalidRequest("bad".into()).is_recoverable());
    }

    #[test]
    fn test_error_constructors() {
        let err = ProxyError::connection_failed("example.com", "refused");
        assert!(matches!(err, ProxyError::ConnectionFailed { .. }));
    }
}
