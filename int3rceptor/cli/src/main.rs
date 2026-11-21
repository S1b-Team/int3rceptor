// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                INT3RCEPTOR                                ║
// ║                   Copyright (c) 2025 S1BGr0uP                             ║
// ║                        All Rights Reserved                                ║
// ╠═══════════════════════════════════════════════════════════════════════════╣
// ║  PROPRIETARY SOFTWARE - Unauthorized copying, modification, distribution, ║
// ║  or use of this software is strictly prohibited without explicit written  ║
// ║  permission from S1BGr0uP. See LICENSE file for full terms.               ║
// ║                                                                           ║
// ║  Build Fingerprint: {BUILD_ID}                                            ║
// ║  License: https://github.com/S1b-Team/int3rceptor/blob/main/LICENSE      ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

use clap::Parser;
use interceptor_api::audit::AuditLogger;
use interceptor_api::csrf::CsrfProtection;
use interceptor_api::ip_filter::{IpFilter, IpFilterConfig};
use interceptor_api::models::{AppSettings, ProxyConfig, UiConfig};
use interceptor_core::connection_pool::ConnectionPool;
use interceptor_core::plugin::config::PluginSystemConfig;
use interceptor_core::plugin::manager::PluginManager;
use interceptor_core::tls::TlsInterceptor;
use interceptor_core::{
    capture::RequestCapture, cert_manager::CertManager, rules::RuleEngine, storage::CaptureStorage,
    Intruder, ProjectManager, Scanner, ScopeManager, WsCapture,
};
use interceptor_core::{InterceptQueue, ProxyController};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

// Build-time watermark - DO NOT REMOVE
const BUILD_FINGERPRINT: &str = env!("BUILD_FINGERPRINT");
const COPYRIGHT_NOTICE: &str = concat!(
    "Int3rceptor v",
    env!("CARGO_PKG_VERSION"),
    " - Copyright (c) 2025 S1BGr0uP. All Rights Reserved."
);
const LICENSE_URL: &str = "https://github.com/S1b-Team/int3rceptor/blob/main/LICENSE";

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

    // Display copyright watermark
    println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
    println!(
        "║{:^75}║",
        concat!("INT3RCEPTOR v", env!("CARGO_PKG_VERSION"))
    );
    println!("║                   Copyright (c) 2025 S1BGr0uP                             ║");
    println!("║                        All Rights Reserved                                ║");
    println!("╠═══════════════════════════════════════════════════════════════════════════╣");
    println!("║  PROPRIETARY SOFTWARE - Licensed under Proprietary License                ║");
    println!(
        "║  Build: {}                                    ║",
        BUILD_FINGERPRINT
    );
    println!("║  License: {}  ║", LICENSE_URL);
    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");

    info!("{}", COPYRIGHT_NOTICE);

    // Initialize license manager
    let mut license_manager = interceptor_core::LicenseManager::new();
    license_manager.load_license()?;

    let license_tier = license_manager.tier();
    if let Some(license) = license_manager.license() {
        println!(
            "📜 License: {} - {}",
            match license_tier {
                interceptor_core::LicenseTier::Free => "FREE (Personal Use)",
                interceptor_core::LicenseTier::Professional => "PROFESSIONAL",
                interceptor_core::LicenseTier::Enterprise => "ENTERPRISE",
            },
            license.licensee
        );

        if let Some(days) = license.days_until_expiration() {
            if days < 30 {
                println!("⚠️  License expires in {} days", days);
            }
        }

        println!("   Max Connections: {}", license_tier.max_connections());
        println!("   Max RPS: {}\n", license_tier.max_rps());
    }

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
    let api_token = std::env::var("INTERCEPTOR_API_TOKEN")
        .ok()
        .filter(|token| !token.is_empty())
        .map(Arc::new);
    let dev_mode = std::env::var("INTERCEPTOR_DEV_MODE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    check_api_exposure(api_token.is_some(), &cli.api, dev_mode).map_err(|e| anyhow::anyhow!(e))?;
    let max_body_bytes = std::env::var("INTERCEPTOR_MAX_BODY_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2 * 1024 * 1024);
    let max_concurrency = std::env::var("INTERCEPTOR_MAX_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(64);

    // Initialize audit logging
    let audit_logger = if let Ok(audit_path) = std::env::var("AUDIT_LOG_PATH") {
        match AuditLogger::new(&audit_path) {
            Ok(logger) => {
                info!("Audit logging enabled: {}", audit_path);
                Some(Arc::new(logger))
            }
            Err(e) => {
                tracing::warn!("Failed to initialize audit logger: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Initialize CSRF protection
    let csrf_protection = if std::env::var("CSRF_PROTECTION")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
    {
        let secret =
            std::env::var("CSRF_SECRET").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
        info!("CSRF protection enabled");
        Some(Arc::new(CsrfProtection::new(secret)))
    } else {
        None
    };

    // Initialize IP filtering
    let ip_filter = Arc::new(IpFilter::new(IpFilterConfig::default()));

    let settings = Arc::new(RwLock::new(AppSettings {
        proxy: ProxyConfig {
            port: cli.listen.port(),
            host: cli.listen.ip().to_string(),
            intercept_https: true,
            http2: true,
        },
        ui: UiConfig {
            theme: "cyberpunk".to_string(),
            animations: true,
            notifications: true,
        },
    }));

    let plugin_manager = Arc::new(PluginManager::new(PluginSystemConfig::default()));
    // Try to load plugins from default directory
    if let Err(e) = plugin_manager.load_all() {
        tracing::warn!("Failed to load plugins: {}", e);
    }

    if let Some(path) = cli.export_ca.as_ref() {
        cert_manager.export_ca_cert(path)?;
        println!("CA certificate exported to {}", path.display());
        return Ok(());
    }

    let project_manager = Arc::new(ProjectManager::new(Some(storage.clone())));
    let scanner = Arc::new(Scanner::new());

    // Initialize license manager
    let mut license_manager = interceptor_core::license::LicenseManager::new();
    if let Err(e) = license_manager.load_license() {
        tracing::warn!("Failed to load license: {}", e);
    }
    let license_manager = Arc::new(license_manager);

    let intercept = Arc::new(InterceptQueue::new());
    let proxy_controller = Arc::new(ProxyController::new(
        cli.listen,
        capture.clone(),
        rules.clone(),
        scope.clone(),
        Some(tls.clone()),
        Some(plugin_manager.clone()),
        Some(scanner.clone()),
        intercept.clone(),
    ));
    proxy_controller.set_ws_capture(ws_capture.clone());

    // Start proxy immediately (CLI default); API can still stop/start it.
    proxy_controller
        .start()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let api_state = interceptor_api::state::AppState {
        capture: capture.clone(),
        cert_manager: cert_manager.clone(),
        pool: ConnectionPool::new(),
        rules: rules.clone(),
        scope: scope.clone(),
        intruder: intruder.clone(),
        scanner: scanner.clone(),
        ws_capture: ws_capture.clone(),
        project_manager,
        api_token,
        max_body_bytes,
        max_concurrency,
        audit_logger,
        csrf_protection,
        ip_filter,
        settings,
        plugin_manager: plugin_manager.clone(),
        license_manager,
        proxy: proxy_controller,
        intercept,
    };
    let api_addr = cli.api;
    interceptor_api::serve(api_state, api_addr).await?;

    info!("Shutdown complete");
    Ok(())
}

/// Refuses to expose an unauthenticated API beyond the loopback interface.
fn check_api_exposure(
    has_token: bool,
    addr: &std::net::SocketAddr,
    dev_mode: bool,
) -> Result<(), String> {
    if has_token {
        return Ok(());
    }
    if addr.ip().is_loopback() {
        tracing::warn!(
            "INTERCEPTOR_API_TOKEN is not set: the API on {addr} accepts requests without authentication"
        );
        return Ok(());
    }
    if dev_mode {
        tracing::warn!("DEV MODE: serving the API on {addr} without authentication");
        return Ok(());
    }
    Err(format!(
        "refusing to serve the API on {addr} without INTERCEPTOR_API_TOKEN \
         (set a token, bind to 127.0.0.1, or set INTERCEPTOR_DEV_MODE=1)"
    ))
}

#[cfg(test)]
mod exposure_tests {
    use super::check_api_exposure;

    fn addr(s: &str) -> std::net::SocketAddr {
        s.parse().unwrap()
    }

    #[test]
    fn a_token_always_allows_serving() {
        assert!(check_api_exposure(true, &addr("0.0.0.0:3000"), false).is_ok());
    }

    #[test]
    fn loopback_without_a_token_is_allowed() {
        assert!(check_api_exposure(false, &addr("127.0.0.1:3000"), false).is_ok());
        assert!(check_api_exposure(false, &addr("[::1]:3000"), false).is_ok());
    }

    #[test]
    fn exposed_without_a_token_is_refused_unless_dev_mode() {
        assert!(check_api_exposure(false, &addr("0.0.0.0:3000"), false).is_err());
        assert!(check_api_exposure(false, &addr("192.168.1.5:3000"), false).is_err());
        assert!(check_api_exposure(false, &addr("0.0.0.0:3000"), true).is_ok());
    }
}
