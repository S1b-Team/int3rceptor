pub mod auth;
pub mod models;
pub mod routes;
pub mod state;
pub mod websocket;

use auth::require_auth;
use axum::{middleware, routing::get, Extension, Router};
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};
use tracing::info;

pub fn build_router(state: Arc<AppState>) -> Router {
    routes::router()
        .route("/ws", get(websocket::ws_route))
        .layer(CorsLayer::permissive())
        .layer(Extension(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .layer(RequestBodyLimitLayer::new(state.max_body_bytes))
        .layer(ConcurrencyLimitLayer::new(state.max_concurrency))
}

pub async fn serve(state: AppState, addr: SocketAddr) -> anyhow::Result<()> {
    let shared_state = Arc::new(state);
    let app = build_router(shared_state);
    info!("API listening on http://{addr}");
    let listener = TcpListener::bind(addr).await?;
    let service = app.into_make_service();
    axum::serve(listener, service).await?;
    Ok(())
}
