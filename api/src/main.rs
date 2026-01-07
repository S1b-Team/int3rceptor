use interceptor_api::audit::{AuditCategory, AuditLogger, AuditSeverity};
use interceptor_api::csrf::CsrfProtection;
use interceptor_api::ip_filter::{IpFilter, IpFilterConfig};
use interceptor_api::models::{AppSettings, ProxyConfig, UiConfig};
use interceptor_api::serve;
use interceptor_api::state::AppState;
use interceptor_core::capture::RequestCapture;
use interceptor_core::cert_manager::CertManager;
use interceptor_core::connection_pool::ConnectionPool;
use interceptor_core::plugin::config::PluginSystemConfig;
use interceptor_core::plugin::manager::PluginManager;
use interceptor_core::rules::RuleEngine;
use interceptor_core::storage::CaptureStorage;
use interceptor_core::{Intruder, ProjectManager, Scanner, ScopeManager, WsCapture};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _sentry_guard = interceptor_core::telemetry::sentry::init_sentry();
    sentry::configure_scope(|scope| {
        scope.set_tag("component", "api");
        scope.set_tag(
            "tier",
            std::env::var("SENTRY_TELEMETRY_TIER").unwrap_or_else(|_| "0".to_string()),
        );
        scope.set_tag("os", std::env::consts::OS);
        scope.set_tag("arch", std::env::consts::ARCH);
    });
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

    // P2 Security Hardening: Mandatory API Authentication in Production
    // Allow dev mode with INTERCEPTOR_DEV_MODE=1 for testing
    let dev_mode = std::env::var("INTERCEPTOR_DEV_MODE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    let api_token = if dev_mode {
        tracing::warn!("⚠️  DEV MODE ENABLED - API authentication is optional");
        tracing::warn!("⚠️  For production: unset INTERCEPTOR_DEV_MODE or set it to 0");
        std::env::var("INTERCEPTOR_API_TOKEN").ok().map(Arc::new)
    } else {
        // Production mode: API token is REQUIRED
        match std::env::var("INTERCEPTOR_API_TOKEN") {
            Ok(token) => {
                info!("✓ API authentication enabled (production mode)");
                Some(Arc::new(token))
            }
            Err(_) => {
                eprintln!();
                eprintln!("╔════════════════════════════════════════════════════════════════╗");
                eprintln!("║ SECURITY ERROR: API token required in production mode          ║");
                eprintln!("║ Set INTERCEPTOR_API_TOKEN environment variable or enable       ║");
                eprintln!("║ development mode with: INTERCEPTOR_DEV_MODE=1                 ║");
                eprintln!("╚════════════════════════════════════════════════════════════════╝");
                eprintln!();
                panic!(
                    "INTERCEPTOR_API_TOKEN is required in production mode. \
                     Set INTERCEPTOR_DEV_MODE=1 to enable development mode without auth."
                );
            }
        }
    };
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

    // Log server startup
    if let Some(ref logger) = audit_logger {
        logger.log(
            interceptor_api::audit::AuditEntry::new(
                AuditSeverity::Info,
                AuditCategory::SystemChange,
                "server_startup",
                "system",
            )
            .with_details(serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "tls_enabled": std::env::var("TLS_CERT_PATH").is_ok(),
            })),
        );
    }

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
        info!("CSRF protection disabled");
        None
    };

    // Initialize IP filtering
    let ip_filter_config = if let Ok(config_str) = std::env::var("IP_FILTER_CONFIG") {
        match serde_json::from_str::<IpFilterConfig>(&config_str) {
            Ok(config) => {
                info!("IP filter configured from environment");
                config
            }
            Err(e) => {
                tracing::warn!("Failed to parse IP filter config: {}", e);
                IpFilterConfig::default()
            }
        }
    } else {
        IpFilterConfig::default()
    };

    let ip_filter = Arc::new(IpFilter::new(ip_filter_config));
    if ip_filter.get_config().mode != "off" {
        info!("IP filtering enabled");
    }

    let settings = Arc::new(RwLock::new(AppSettings {
        proxy: ProxyConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
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

    let mut license_manager = interceptor_core::license::LicenseManager::new();
    if let Err(e) = license_manager.load_license() {
        tracing::warn!("Failed to load license: {}", e);
    }
    let license_manager = Arc::new(license_manager);

    let project_manager = Arc::new(ProjectManager::new(Some(storage.clone())));

    let state = AppState {
        capture,
        cert_manager,
        pool: ConnectionPool::new(),
        rules,
        scope,
        intruder,
        scanner: Arc::new(Scanner::new()),
        ws_capture,
        project_manager,
        api_token,
        max_body_bytes,
        max_concurrency,
        audit_logger,
        csrf_protection,
        ip_filter,
        settings,
        plugin_manager,
        license_manager,
    };

    let addr: SocketAddr = std::env::var("API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".into())
        .parse()?;

    serve(state, addr).await
}
