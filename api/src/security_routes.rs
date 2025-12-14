use crate::{
    audit::{AuditCategory, AuditEntry, AuditSeverity},
    ip_filter::IpFilterConfig,
    state::AppState,
};
use axum::extract::{ConnectInfo, Extension, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::net::SocketAddr;
use std::sync::Arc;

/// Get IP filter configuration
pub async fn get_ip_filter_config(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    Json(state.ip_filter.get_config())
}

/// Update IP filter configuration
pub async fn set_ip_filter_config(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(state): Extension<Arc<AppState>>,
    Json(config): Json<IpFilterConfig>,
) -> impl IntoResponse {
    state.ip_filter.set_config(config.clone());

    // Log configuration change
    if let Some(ref logger) = state.audit_logger {
        logger.log(
            AuditEntry::new(
                AuditSeverity::Warning,
                AuditCategory::Configuration,
                "ip_filter_config_updated",
                "admin",
            )
            .with_ip(addr.ip())
            .with_details(serde_json::json!({
                "mode": config.mode,
                "allowed_count": config.allowed_ips.len(),
                "blocked_count": config.blocked_ips.len(),
            })),
        );
    }

    StatusCode::NO_CONTENT
}

/// Add IP to allowlist
pub async fn add_allowed_ip(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ip_str = match payload.get("ip").and_then(|v| v.as_str()) {
        Some(ip) => ip,
        None => return (StatusCode::BAD_REQUEST, "Missing 'ip' field").into_response(),
    };

    let ip = match ip_str.parse() {
        Ok(ip) => ip,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid IP address").into_response(),
    };

    state.ip_filter.add_allowed_ip(ip);

    // Log IP addition
    if let Some(ref logger) = state.audit_logger {
        logger.log(
            AuditEntry::new(
                AuditSeverity::Warning,
                AuditCategory::Configuration,
                "ip_added_to_allowlist",
                "admin",
            )
            .with_ip(addr.ip())
            .with_resource(ip_str),
        );
    }

    StatusCode::CREATED.into_response()
}

/// Add IP to blocklist
pub async fn add_blocked_ip(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ip_str = match payload.get("ip").and_then(|v| v.as_str()) {
        Some(ip) => ip,
        None => return (StatusCode::BAD_REQUEST, "Missing 'ip' field").into_response(),
    };

    let ip = match ip_str.parse() {
        Ok(ip) => ip,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid IP address").into_response(),
    };

    state.ip_filter.add_blocked_ip(ip);

    // Log IP addition
    if let Some(ref logger) = state.audit_logger {
        logger.log(
            AuditEntry::new(
                AuditSeverity::Critical,
                AuditCategory::SecurityEvent,
                "ip_added_to_blocklist",
                "admin",
            )
            .with_ip(addr.ip())
            .with_resource(ip_str),
        );
    }

    StatusCode::CREATED.into_response()
}

/// Rotate audit log
pub async fn rotate_audit_log(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(ref logger) = state.audit_logger {
        // Log rotation event before rotating
        logger.log(
            AuditEntry::new(
                AuditSeverity::Info,
                AuditCategory::SystemChange,
                "audit_log_rotation_requested",
                "admin",
            )
            .with_ip(addr.ip()),
        );

        match logger.rotate() {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to rotate log: {}", e),
            )
                .into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Audit logging not enabled").into_response()
    }
}

/// Get audit log info
pub async fn get_audit_log_info(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    if let Some(ref logger) = state.audit_logger {
        Json(serde_json::json!({
            "enabled": true,
            "log_path": logger.log_path().to_string_lossy(),
        }))
        .into_response()
    } else {
        Json(serde_json::json!({
            "enabled": false,
        }))
        .into_response()
    }
}
