pub mod audit;
pub mod auth;
pub mod csrf;
pub mod error;
pub mod ip_filter;
pub mod models;
pub mod routes;
pub mod security_routes;
pub mod state;
pub mod tls;
pub mod websocket;

use auth::{rate_limit, request_size_limit, require_auth, security_headers_middleware};
use axum::{middleware, routing::get, Extension, Router};
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};
use tracing::info;

pub fn build_router(state: Arc<AppState>) -> Router {
    let mut router = routes::router()
        .route("/ws", get(websocket::ws_route))
        // Add CSRF token endpoint
        .route("/api/csrf-token", get(csrf::generate_csrf_token));

    // Apply IP filter middleware if configured
    if state.ip_filter.get_config().mode != "off" {
        router = router.layer(middleware::from_fn_with_state(
            state.ip_filter.clone(),
            ip_filter::ip_filter_middleware,
        ));
    }

    // Apply CSRF protection middleware if configured
    if let Some(ref csrf_protection) = state.csrf_protection {
        router = router.layer(middleware::from_fn_with_state(
            csrf_protection.clone(),
            csrf::csrf_protection_middleware,
        ));
    }

    router
        // Innermost layers (run first)
        .layer(Extension(state.clone()))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            request_size_limit,
        ))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth))
        // Rate limiting (needs ConnectInfo)
        .layer(middleware::from_fn(rate_limit))
        // CORS
        .layer(CorsLayer::permissive())
        // Security headers (outermost, runs on response)
        .layer(middleware::from_fn(security_headers_middleware))
        // Tower layers
        .layer(RequestBodyLimitLayer::new(state.max_body_bytes))
        .layer(ConcurrencyLimitLayer::new(state.max_concurrency))
}

pub async fn serve(state: AppState, addr: SocketAddr) -> anyhow::Result<()> {
    let shared_state = Arc::new(state);
    let app = build_router(shared_state);

    // Check if TLS is configured
    if let Some(tls_config) = tls::TlsConfig::from_env() {
        info!("TLS configuration detected, starting HTTPS server");
        tls::serve_with_tls(app, addr, tls_config).await
    } else {
        info!("No TLS configuration found, starting HTTP server");
        info!("API listening on http://{addr}");
        let listener = TcpListener::bind(addr).await?;
        let service = app.into_make_service_with_connect_info::<SocketAddr>();
        axum::serve(listener, service).await?;
        Ok(())
    }
}
