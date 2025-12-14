//! API Error handling
//!
//! Provides structured error responses for the REST API with:
//! - Consistent JSON error format
//! - Error codes from interceptor-core
//! - HTTP status code mapping

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use interceptor_core::error::{ErrorCode, ProxyError};
use serde::Serialize;

/// Structured API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Error code for client handling
    pub code: u16,

    /// Error code string identifier
    pub error: &'static str,

    /// Human-readable error message
    pub message: String,

    /// Optional additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    /// Create a new API error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code.code(),
            error: code.as_str(),
            message: message.into(),
            details: None,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, format!("{} not found", resource.into()))
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    /// Create a bad request error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::HttpInvalidRequest, message)
    }

    /// Create from a ProxyError
    pub fn from_proxy_error(err: &ProxyError) -> Self {
        Self::new(err.code(), err.to_string())
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        error_code_to_status(ErrorCode::from_code(self.code))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        (status, Json(self)).into_response()
    }
}

impl From<ProxyError> for ApiError {
    fn from(err: ProxyError) -> Self {
        Self::from_proxy_error(&err)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::bad_request(format!("JSON error: {}", err))
    }
}

/// Map ErrorCode to HTTP StatusCode
fn error_code_to_status(code: ErrorCode) -> StatusCode {
    match code {
        // 4xx Client Errors
        ErrorCode::NotFound => StatusCode::NOT_FOUND,
        ErrorCode::AlreadyExists => StatusCode::CONFLICT,
        ErrorCode::InvalidConfig => StatusCode::BAD_REQUEST,
        ErrorCode::HttpInvalidRequest => StatusCode::BAD_REQUEST,
        ErrorCode::HttpInvalidUri => StatusCode::BAD_REQUEST,
        ErrorCode::HttpInvalidHeader => StatusCode::BAD_REQUEST,
        ErrorCode::HttpBodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        ErrorCode::RuleInvalid => StatusCode::BAD_REQUEST,
        ErrorCode::RuleNotFound => StatusCode::NOT_FOUND,
        ErrorCode::RuleRegexInvalid => StatusCode::BAD_REQUEST,
        ErrorCode::ScopeInvalid => StatusCode::BAD_REQUEST,
        ErrorCode::ScopePatternInvalid => StatusCode::BAD_REQUEST,
        ErrorCode::IntruderJobNotFound => StatusCode::NOT_FOUND,
        ErrorCode::IntruderInvalidPayload => StatusCode::BAD_REQUEST,
        ErrorCode::LicenseInvalid => StatusCode::FORBIDDEN,
        ErrorCode::LicenseExpired => StatusCode::FORBIDDEN,
        ErrorCode::LicenseFeatureDisabled => StatusCode::FORBIDDEN,

        // 5xx Server Errors
        ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::IoError => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::DatabaseConnection => StatusCode::SERVICE_UNAVAILABLE,
        ErrorCode::DatabaseQuery => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::DatabaseMigration => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::StorageFull => StatusCode::INSUFFICIENT_STORAGE,
        ErrorCode::LockPoisoned => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::LockTimeout => StatusCode::SERVICE_UNAVAILABLE,

        // 502/503 for upstream/connection errors
        ErrorCode::ConnectionFailed => StatusCode::BAD_GATEWAY,
        ErrorCode::ConnectionTimeout => StatusCode::GATEWAY_TIMEOUT,
        ErrorCode::ConnectionClosed => StatusCode::BAD_GATEWAY,
        ErrorCode::DnsResolution => StatusCode::BAD_GATEWAY,
        ErrorCode::ProxyUpstreamFailed => StatusCode::BAD_GATEWAY,
        ErrorCode::ProxyConnectFailed => StatusCode::BAD_GATEWAY,

        // Proxy state errors
        ErrorCode::ProxyNotRunning => StatusCode::SERVICE_UNAVAILABLE,
        ErrorCode::ProxyAlreadyRunning => StatusCode::CONFLICT,

        // TLS errors
        ErrorCode::TlsHandshake => StatusCode::BAD_GATEWAY,
        ErrorCode::TlsCertificate => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::TlsVerification => StatusCode::BAD_GATEWAY,
        ErrorCode::TlsProtocol => StatusCode::BAD_GATEWAY,

        // Rate limiting
        ErrorCode::IntruderRateLimited => StatusCode::TOO_MANY_REQUESTS,

        // WebSocket errors
        ErrorCode::WebSocketParse => StatusCode::BAD_REQUEST,
        ErrorCode::WebSocketClosed => StatusCode::GONE,
        ErrorCode::WebSocketProtocol => StatusCode::BAD_REQUEST,

        // HTTP parsing
        ErrorCode::HttpParse => StatusCode::BAD_REQUEST,
        ErrorCode::HttpInvalidResponse => StatusCode::BAD_GATEWAY,

        // Fallback
        ErrorCode::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::RuleConflict => StatusCode::CONFLICT,
    }
}

/// Result type for API handlers
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_serialization() {
        let err = ApiError::not_found("request 123");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("NOT_FOUND"));
        assert!(json.contains("1003"));
    }

    #[test]
    fn test_status_code_mapping() {
        let err = ApiError::not_found("test");
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);

        let err = ApiError::internal("oops");
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
