//! CORS policy for the API.
//!
//! The API controls an intercepting proxy, so any web page the user visits must
//! not be able to talk to it. Only the desktop shell and the local dev server are
//! allowed by default; extra origins are opt-in through `INTERCEPTOR_CORS_ORIGINS`.

use axum::http::{header, HeaderName, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::warn;

/// Origins used by the Tauri desktop app (per platform) and its Vite dev server.
pub const DEFAULT_ORIGINS: &[&str] = &[
    "tauri://localhost",
    "http://tauri.localhost",
    "https://tauri.localhost",
    "http://localhost:5173",
    "http://127.0.0.1:5173",
];

/// Reads `INTERCEPTOR_CORS_ORIGINS` (comma separated). Empty or unset means the
/// defaults; the single value `*` allows every origin and must be chosen explicitly.
pub fn origins_from_env() -> Vec<String> {
    parse_origins(std::env::var("INTERCEPTOR_CORS_ORIGINS").ok().as_deref())
}

pub fn parse_origins(value: Option<&str>) -> Vec<String> {
    let configured: Vec<String> = value
        .unwrap_or_default()
        .split(',')
        .map(|origin| origin.trim().trim_end_matches('/').to_string())
        .filter(|origin| !origin.is_empty())
        .collect();
    if configured.is_empty() {
        DEFAULT_ORIGINS.iter().map(|o| o.to_string()).collect()
    } else {
        configured
    }
}

pub fn layer(origins: &[String]) -> CorsLayer {
    let allow_origin = if origins.iter().any(|origin| origin == "*") {
        warn!("INTERCEPTOR_CORS_ORIGINS=* allows any web page to call the API");
        AllowOrigin::any()
    } else {
        let values: Vec<HeaderValue> = origins
            .iter()
            .filter_map(|origin| match HeaderValue::from_str(origin) {
                Ok(value) => Some(value),
                Err(_) => {
                    warn!("Ignoring invalid CORS origin: {origin}");
                    None
                }
            })
            .collect();
        AllowOrigin::list(values)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("x-csrf-token"),
        ])
        .expose_headers(Any)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::{routing::get, Router};
    use tower::util::ServiceExt;

    fn app(origins: &[&str]) -> Router {
        let origins: Vec<String> = origins.iter().map(|o| o.to_string()).collect();
        Router::new()
            .route("/ping", get(|| async { "pong" }))
            .layer(layer(&origins))
    }

    async fn allowed_origin(app: Router, origin: &str) -> Option<String> {
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ping")
                    .header("origin", origin)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        response
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap().to_string())
    }

    #[test]
    fn unset_or_blank_uses_the_defaults() {
        assert_eq!(parse_origins(None), parse_origins(Some("  , ")));
        assert!(parse_origins(None).contains(&"tauri://localhost".to_string()));
    }

    #[test]
    fn configured_origins_replace_the_defaults_and_are_trimmed() {
        let origins = parse_origins(Some(" https://a.example/ , https://b.example "));
        assert_eq!(origins, vec!["https://a.example", "https://b.example"]);
    }

    #[tokio::test]
    async fn defaults_allow_the_desktop_shell_only() {
        let defaults: Vec<&str> = DEFAULT_ORIGINS.to_vec();
        assert_eq!(
            allowed_origin(app(&defaults), "tauri://localhost").await,
            Some("tauri://localhost".to_string())
        );
        assert_eq!(
            allowed_origin(app(&defaults), "https://evil.example").await,
            None
        );
    }

    #[tokio::test]
    async fn star_is_an_explicit_opt_in() {
        assert_eq!(
            allowed_origin(app(&["*"]), "https://anything.example").await,
            Some("*".to_string())
        );
    }

    #[tokio::test]
    async fn preflight_from_a_foreign_origin_gets_no_allowance() {
        let response = app(&["tauri://localhost"])
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/ping")
                    .header("origin", "https://evil.example")
                    .header("access-control-request-method", "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response
            .headers()
            .get("access-control-allow-origin")
            .is_none());
    }
}
