use interceptor_api::serve;
use interceptor_api::state::AppState;
use interceptor_core::capture::RequestCapture;
use interceptor_core::cert_manager::CertManager;
use interceptor_core::connection_pool::ConnectionPool;
use interceptor_core::rules::RuleEngine;
use interceptor_core::storage::CaptureStorage;
use interceptor_core::{Intruder, ScopeManager, WsCapture};
use std::{net::SocketAddr, sync::Arc};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let db_path =
        std::env::var("INTERCEPTOR_DB_PATH").unwrap_or_else(|_| "data/interceptor.sqlite".into());
    let storage = Arc::new(CaptureStorage::new(db_path)?);
    let capture = Arc::new(RequestCapture::with_storage(10_000, Some(storage.clone())));

    let cert_manager = Arc::new(CertManager::new()?);
    let rules = Arc::new(RuleEngine::new());
    info!("Initialized RuleEngine");

    let scope = Arc::new(ScopeManager::new());
    info!("Initialized ScopeManager");

    let intruder = Arc::new(Intruder::new());
    info!("Initialized Intruder");

    let ws_capture = Arc::new(WsCapture::new(10_000));
    info!("Initialized WebSocket Capture");
    let api_token = std::env::var("INTERCEPTOR_API_TOKEN").ok().map(Arc::new);
    let max_body_bytes = std::env::var("INTERCEPTOR_MAX_BODY_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2 * 1024 * 1024);
    let max_concurrency = std::env::var("INTERCEPTOR_MAX_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(64);
    let state = AppState {
        capture,
        cert_manager,
        pool: ConnectionPool::new(),
        rules,
        scope,
        intruder,
        ws_capture,
        api_token,
        max_body_bytes,
        max_concurrency,
    };
    let addr: SocketAddr = std::env::var("API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".into())
        .parse()?;
    serve(state, addr).await
}
