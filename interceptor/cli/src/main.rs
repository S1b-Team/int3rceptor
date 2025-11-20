use clap::Parser;
use interceptor_core::connection_pool::ConnectionPool;
use interceptor_core::proxy::ProxyServer;
use interceptor_core::tls::TlsInterceptor;
use interceptor_core::{
    capture::RequestCapture, cert_manager::CertManager, rules::RuleEngine, storage::CaptureStorage, Intruder, ScopeManager, WsCapture,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "interceptor")]
#[command(about = "HTTP/HTTPS intercepting proxy")]
struct Cli {
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    listen: SocketAddr,
    #[arg(short, long, default_value = "127.0.0.1:3000")]
    api: SocketAddr,
    #[arg(long)]
    export_ca: Option<PathBuf>,
    #[arg(short, long, default_value = "info")]
    verbosity: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| cli.verbosity.clone());
    tracing_subscriber::fmt().with_env_filter(filter).init();

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
    let tls = Arc::new(TlsInterceptor::new(cert_manager.clone())?);
    let api_token = std::env::var("INTERCEPTOR_API_TOKEN").ok().map(Arc::new);
    let max_body_bytes = std::env::var("INTERCEPTOR_MAX_BODY_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2 * 1024 * 1024);
    let max_concurrency = std::env::var("INTERCEPTOR_MAX_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(64);

    if let Some(path) = cli.export_ca.as_ref() {
        cert_manager.export_ca_cert(path)?;
        println!("CA certificate exported to {}", path.display());
        return Ok(());
    }

    let api_state = interceptor_api::state::AppState {
        capture: capture.clone(),
        cert_manager: cert_manager.clone(),
        pool: ConnectionPool::new(),
        rules: rules.clone(),
        scope: scope.clone(),
        intruder: intruder.clone(),
        ws_capture: ws_capture.clone(),
        api_token,
        max_body_bytes,
        max_concurrency,
    };
    let api_addr = cli.api;
    let api_task = tokio::spawn(async move { interceptor_api::serve(api_state, api_addr).await });

    let proxy = ProxyServer::new(
        cli.listen,
        capture.clone(),
        rules.clone(),
        scope.clone(),
        Some(tls),
    );
    let proxy_task = tokio::spawn(async move { proxy.run().await });

    let (api_res, proxy_res) = tokio::try_join!(api_task, proxy_task)?;
    api_res?;
    proxy_res?;

    info!("Shutdown complete");
    Ok(())
}
