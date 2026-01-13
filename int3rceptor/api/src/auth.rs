use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use crate::state::AppState;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{header, HeaderName, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use interceptor_core::security::{constant_time_compare, headers as security_headers, RateLimiter};
use once_cell::sync::Lazy;

/// Global rate limiter: 100 requests per minute per IP
static RATE_LIMITER: Lazy<RateLimiter> =
    Lazy::new(|| RateLimiter::new(100, Duration::from_secs(60)));

/// Authentication middleware with timing-safe comparison
/// P2 Security Hardening: Improved logging and validation
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    if let Some(expected_token) = &state.api_token {
        let bearer_prefix = "Bearer ";
        let ip = addr.ip();
        let method = req.method().clone();
        let path = req.uri().to_string();

        // Check Authorization header (Bearer token)
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok());

        let bearer_auth = auth_header
            .filter(|value| value.starts_with(bearer_prefix))
            .map(|value| &value[bearer_prefix.len()..])
            .map(|token| constant_time_compare(token, expected_token))
            .unwrap_or(false);

        // Check x-api-key header
        let api_key = req
            .headers()
            .get("x-api-key")
            .and_then(|value| value.to_str().ok());

        let api_key_auth = api_key
            .map(|token| constant_time_compare(token, expected_token))
            .unwrap_or(false);

        if !bearer_auth && !api_key_auth {
            // Log authentication failure with details
            let auth_method = if auth_header.is_some() {
                "Authorization header (invalid)"
            } else if api_key.is_some() {
                "x-api-key header (invalid)"
            } else {
                "none"
            };

            tracing::warn!(
                ip = %ip,
                method = %method,
                path = %path,
                auth_method = auth_method,
                "API authentication failed - access denied"
            );

            return (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer realm=\"api\"")],
                "Unauthorized: Invalid or missing API token",
            )
                .into_response();
        }

        // Log successful authentication
        tracing::debug!(
            ip = %ip,
            method = %method,
            path = %path,
            "API authentication successful"
        );
    }
    next.run(req).await
}

/// Rate limiting middleware
pub async fn rate_limit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let ip = addr.ip();
    let result = RATE_LIMITER.check(ip);

    match result {
        interceptor_core::RateLimitResult::Allowed { remaining } => {
            let mut response = next.run(req).await;

            // Add rate limit headers
            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Limit", "100".parse().unwrap());
            headers.insert(
                "X-RateLimit-Remaining",
                remaining.to_string().parse().unwrap(),
            );

            response
        }
        interceptor_core::RateLimitResult::Limited { retry_after, .. } => {
            tracing::warn!(
                ip = %ip,
                "Rate limit exceeded"
            );
            (
                StatusCode::TOO_MANY_REQUESTS,
                [
                    ("X-RateLimit-Limit", "100"),
                    ("X-RateLimit-Remaining", "0"),
                    ("Retry-After", &retry_after.as_secs().to_string()),
                ],
                "Rate limit exceeded. Please slow down.",
            )
                .into_response()
        }
    }
}

/// Security headers middleware
pub async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;

    // Add security headers
    let headers = response.headers_mut();
    for (name, value) in security_headers::all() {
        if let (Ok(name), Ok(value)) = (name.parse::<HeaderName>(), value.parse::<HeaderValue>()) {
            headers.insert(name, value);
        }
    }

    response
}

/// Request size limit middleware
pub async fn request_size_limit(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    // Check Content-Length header
    if let Some(content_length) = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
    {
        if content_length > state.max_body_bytes {
            tracing::warn!(
                content_length,
                max = state.max_body_bytes,
                "Request body too large"
            );
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                format!(
                    "Request body exceeds maximum size of {} bytes",
                    state.max_body_bytes
                ),
            )
                .into_response();
        }
    }

    next.run(req).await
}

/// Get rate limiter stats
pub fn rate_limiter_stats() -> interceptor_core::security::RateLimiterStats {
    RATE_LIMITER.stats()
}

/// Cleanup old rate limiter entries
pub fn rate_limiter_cleanup() {
    RATE_LIMITER.cleanup(Duration::from_secs(300));
}
