use interceptor_core::{
    capture::RequestCapture, cert_manager::CertManager, connection_pool::ConnectionPool,
    rules::RuleEngine, Intruder, ScopeManager, WsCapture,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub capture: Arc<RequestCapture>,
    pub cert_manager: Arc<CertManager>,
    pub pool: ConnectionPool,
    pub rules: Arc<RuleEngine>,
    pub scope: Arc<ScopeManager>,
    pub intruder: Arc<Intruder>,
    pub ws_capture: Arc<WsCapture>,
    pub api_token: Option<Arc<String>>,
    pub max_body_bytes: usize,
    pub max_concurrency: usize,
}
