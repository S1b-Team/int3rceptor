use interceptor_core::{
    capture::RequestCapture, cert_manager::CertManager, connection_pool::ConnectionPool,
    plugin::manager::PluginManager, rules::RuleEngine, Intruder, Scanner, ScopeManager, WsCapture,
};
use std::sync::Arc;

use crate::audit::AuditLogger;
use crate::csrf::CsrfProtection;
use crate::ip_filter::IpFilter;
use crate::models::AppSettings;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub capture: Arc<RequestCapture>,
    pub cert_manager: Arc<CertManager>,
    pub pool: ConnectionPool,
    pub rules: Arc<RuleEngine>,
    pub scope: Arc<ScopeManager>,
    pub intruder: Arc<Intruder>,
    pub scanner: Arc<Scanner>,
    pub ws_capture: Arc<WsCapture>,
    pub api_token: Option<Arc<String>>,
    pub max_body_bytes: usize,
    pub max_concurrency: usize,
    // Security features
    pub audit_logger: Option<Arc<AuditLogger>>,
    pub csrf_protection: Option<Arc<CsrfProtection>>,
    pub ip_filter: Arc<IpFilter>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub plugin_manager: Arc<PluginManager>,
}
