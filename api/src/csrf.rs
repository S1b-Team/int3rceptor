use axum::extract::{Request, State};
use axum::http::{header, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CSRF_HEADER: &str = "X-CSRF-Token";
const TOKEN_EXPIRY_SECS: u64 = 3600; // 1 hour

/// CSRF token entry with expiration
#[derive(Debug, Clone)]
struct CsrfTokenEntry {
    token: String,
    created_at: SystemTime,
}

/// CSRF protection manager
pub struct CsrfProtection {
    tokens: RwLock<HashMap<String, CsrfTokenEntry>>,
    secret: String,
}

impl CsrfProtection {
    /// Create a new CSRF protection manager with a secret key
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
            secret: secret.into(),
        }
    }

    /// Generate a new CSRF token for the given client identifier (e.g., session ID or IP)
    pub fn generate_token(&self, client_id: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create token using HMAC-like approach
        let mut hasher = Sha256::new();
        hasher.update(client_id.as_bytes());
        hasher.update(self.secret.as_bytes());
        hasher.update(timestamp.to_string().as_bytes());
        hasher.update(rand::random::<[u8; 16]>());

        let hash = hasher.finalize();
        let token = format!("{:x}", hash);

        // Store token
        if let Ok(mut tokens) = self.tokens.write() {
            tokens.insert(
                client_id.to_string(),
                CsrfTokenEntry {
                    token: token.clone(),
                    created_at: SystemTime::now(),
                },
            );
        }

        token
    }

    /// Validate a CSRF token for the given client identifier
    pub fn validate_token(&self, client_id: &str, provided_token: &str) -> bool {
        let tokens = match self.tokens.read() {
            Ok(t) => t,
            Err(_) => return false,
        };

        if let Some(entry) = tokens.get(client_id) {
            // Check if token is expired
            if let Ok(elapsed) = entry.created_at.elapsed() {
                if elapsed > Duration::from_secs(TOKEN_EXPIRY_SECS) {
                    return false;
                }
            }

            // Timing-safe comparison
            interceptor_core::security::constant_time_compare(provided_token, &entry.token)
        } else {
            false
        }
    }

    /// Remove expired tokens (should be called periodically)
    pub fn cleanup_expired(&self) {
        if let Ok(mut tokens) = self.tokens.write() {
            tokens.retain(|_, entry| {
                entry
                    .created_at
                    .elapsed()
                    .map(|elapsed| elapsed < Duration::from_secs(TOKEN_EXPIRY_SECS))
                    .unwrap_or(false)
            });
        }
    }

    /// Invalidate token for a client
    pub fn invalidate_token(&self, client_id: &str) {
        if let Ok(mut tokens) = self.tokens.write() {
            tokens.remove(client_id);
        }
    }
}

/// Axum middleware for CSRF protection
pub async fn csrf_protection_middleware(
    State(csrf): State<Arc<CsrfProtection>>,
    req: Request,
    next: Next,
) -> Response {
    // CSRF protection only applies to state-changing methods
    if !matches!(req.method(), &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH) {
        return next.run(req).await;
    }

    // Extract client identifier (using IP address as fallback)
    let client_id = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Get CSRF token from header
    let provided_token = req
        .headers()
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok());

    if let Some(token) = provided_token {
        if csrf.validate_token(client_id, token) {
            return next.run(req).await;
        }
    }

    tracing::warn!(
        client_id = %client_id,
        method = %req.method(),
        path = %req.uri(),
        "CSRF token validation failed"
    );

    (
        StatusCode::FORBIDDEN,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"error":"CSRF token missing or invalid"}"#,
    )
        .into_response()
}

/// Endpoint to generate CSRF token
pub async fn generate_csrf_token(
    axum::extract::Extension(state): axum::extract::Extension<std::sync::Arc<crate::state::AppState>>,
    req: Request,
) -> impl IntoResponse {
    if let Some(ref csrf) = state.csrf_protection {
        let client_id = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        let token = csrf.generate_token(client_id);

        axum::Json(serde_json::json!({
            "csrf_token": token,
            "header_name": CSRF_HEADER,
            "expires_in_secs": TOKEN_EXPIRY_SECS,
        }))
        .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"error":"CSRF protection not enabled"}"#,
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_generation_and_validation() {
        let csrf = CsrfProtection::new("test-secret");
        let client_id = "test-client";

        let token = csrf.generate_token(client_id);
        assert!(!token.is_empty());
        assert!(csrf.validate_token(client_id, &token));
    }

    #[test]
    fn test_csrf_token_rejection_wrong_client() {
        let csrf = CsrfProtection::new("test-secret");
        let token = csrf.generate_token("client1");
        assert!(!csrf.validate_token("client2", &token));
    }

    #[test]
    fn test_csrf_token_rejection_wrong_token() {
        let csrf = CsrfProtection::new("test-secret");
        csrf.generate_token("client");
        assert!(!csrf.validate_token("client", "invalid-token"));
    }
}
