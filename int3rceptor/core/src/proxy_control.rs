use crate::capture::RequestCapture;
use crate::connection_pool::ConnectionPool;
use crate::error::Result;
use crate::intercept::InterceptQueue;
use crate::plugin::PluginManager;
use crate::proxy::ProxyServer;
use crate::rules::RuleEngine;
use crate::scanner::Scanner;
use crate::scope::ScopeManager;
use crate::tls::TlsInterceptor;
use crate::websocket::WsCapture;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;
use tracing::{info, warn};

/// Status payload returned by `/api/proxy/status` (and start/stop).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub intercept_https: bool,
    pub start_time: i64,
    pub certificates_generated: u64,
    pub intercept_enabled: bool,
    pub held_count: usize,
}

struct ProxyRuntime {
    shutdown_tx: oneshot::Sender<()>,
    join: tokio::task::JoinHandle<Result<()>>,
}

/// Start / stop the listening proxy from the HTTP API.
#[derive(Clone)]
pub struct ProxyController {
    inner: Arc<ProxyControllerInner>,
}

struct ProxyControllerInner {
    running: AtomicBool,
    addr: Mutex<SocketAddr>,
    start_time: Mutex<i64>,
    runtime: Mutex<Option<ProxyRuntime>>,
    pool: Mutex<Option<ConnectionPool>>,
    capture: Arc<RequestCapture>,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Option<Arc<TlsInterceptor>>,
    plugins: Option<Arc<PluginManager>>,
    scanner: Option<Arc<Scanner>>,
    intercept: Arc<InterceptQueue>,
    intercept_https: AtomicBool,
    certificates_generated: Arc<std::sync::atomic::AtomicU64>,
    ws_capture: Mutex<Arc<WsCapture>>,
}

impl ProxyController {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        addr: SocketAddr,
        capture: Arc<RequestCapture>,
        rules: Arc<RuleEngine>,
        scope: Arc<ScopeManager>,
        tls: Option<Arc<TlsInterceptor>>,
        plugins: Option<Arc<PluginManager>>,
        scanner: Option<Arc<Scanner>>,
        intercept: Arc<InterceptQueue>,
    ) -> Self {
        let intercept_https = tls.is_some();
        Self {
            inner: Arc::new(ProxyControllerInner {
                running: AtomicBool::new(false),
                addr: Mutex::new(addr),
                start_time: Mutex::new(0),
                runtime: Mutex::new(None),
                pool: Mutex::new(None),
                capture,
                rules,
                scope,
                tls,
                plugins,
                scanner,
                intercept,
                intercept_https: AtomicBool::new(intercept_https),
                certificates_generated: Arc::new(std::sync::atomic::AtomicU64::new(0)),
                ws_capture: Mutex::new(Arc::new(WsCapture::new(10_000))),
            }),
        }
    }

    /// Use the same WebSocket capture the HTTP API serves.
    pub fn set_ws_capture(&self, ws_capture: Arc<WsCapture>) {
        *self.inner.ws_capture.lock() = ws_capture;
    }

    /// Replace the upstream client used the next time the listener starts.
    pub fn set_connection_pool(&self, pool: ConnectionPool) {
        *self.inner.pool.lock() = Some(pool);
    }

    pub fn intercept(&self) -> Arc<InterceptQueue> {
        self.inner.intercept.clone()
    }

    pub fn set_addr(&self, addr: SocketAddr) {
        *self.inner.addr.lock() = addr;
    }

    pub fn is_running(&self) -> bool {
        self.inner.running.load(Ordering::SeqCst)
    }

    pub fn status(&self) -> ProxyStatus {
        let addr = *self.inner.addr.lock();
        let intercept = self.inner.intercept.status();
        ProxyStatus {
            running: self.is_running(),
            host: addr.ip().to_string(),
            port: addr.port(),
            tls_enabled: self.inner.tls.is_some(),
            intercept_https: self.inner.intercept_https.load(Ordering::SeqCst),
            start_time: *self.inner.start_time.lock(),
            certificates_generated: self.inner.certificates_generated.load(Ordering::SeqCst),
            intercept_enabled: intercept.enabled,
            held_count: intercept.held_count,
        }
    }

    pub async fn start(&self) -> std::result::Result<ProxyStatus, String> {
        if self.is_running() {
            return Ok(self.status());
        }

        let addr = *self.inner.addr.lock();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let ws_capture = self.inner.ws_capture.lock().clone();
        let proxy = ProxyServer::new(
            addr,
            self.inner.capture.clone(),
            self.inner.rules.clone(),
            self.inner.scope.clone(),
            self.inner.tls.clone(),
            self.inner.plugins.clone(),
            self.inner.scanner.clone(),
        )
        .with_intercept(self.inner.intercept.clone())
        .with_ws_capture(ws_capture);

        let proxy = if let Some(pool) = self.inner.pool.lock().clone() {
            proxy.with_pool(pool)
        } else {
            proxy
        };

        let join = tokio::spawn(async move { proxy.run_until(shutdown_rx).await });

        // Give the listener a moment to bind; surface immediate failures.
        tokio::task::yield_now().await;
        if join.is_finished() {
            match join.await {
                Ok(Ok(())) => {
                    return Err("Proxy exited immediately after start".into());
                }
                Ok(Err(err)) => return Err(err.to_string()),
                Err(err) => return Err(format!("Proxy task failed: {err}")),
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        *self.inner.start_time.lock() = now;
        *self.inner.runtime.lock() = Some(ProxyRuntime { shutdown_tx, join });
        self.inner.running.store(true, Ordering::SeqCst);
        info!(%addr, "Proxy started via controller");
        Ok(self.status())
    }

    pub async fn stop(&self) -> std::result::Result<ProxyStatus, String> {
        if !self.is_running() {
            return Ok(self.status());
        }

        let runtime = self.inner.runtime.lock().take();
        if let Some(runtime) = runtime {
            let _ = runtime.shutdown_tx.send(());
            match runtime.join.await {
                Ok(Ok(())) => {}
                Ok(Err(err)) => warn!(%err, "Proxy stopped with error"),
                Err(err) => warn!(%err, "Proxy join failed"),
            }
        }

        // Release any held requests so client connections can unwind.
        self.inner.intercept.flush_forward();
        self.inner.running.store(false, Ordering::SeqCst);
        *self.inner.start_time.lock() = 0;
        info!("Proxy stopped via controller");
        Ok(self.status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::RequestCapture;
    use crate::rules::RuleEngine;
    use crate::scope::ScopeManager;
    use std::net::{IpAddr, Ipv4Addr};

    fn controller_on_ephemeral() -> ProxyController {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        ProxyController::new(
            addr,
            Arc::new(RequestCapture::new(100)),
            Arc::new(RuleEngine::new()),
            Arc::new(ScopeManager::new()),
            None,
            None,
            None,
            Arc::new(InterceptQueue::new()),
        )
    }

    #[tokio::test]
    async fn status_reports_stopped_by_default() {
        let controller = controller_on_ephemeral();
        let status = controller.status();
        assert!(!status.running);
        assert_eq!(status.port, 0);
        assert!(!status.intercept_enabled);
        assert_eq!(status.held_count, 0);
    }

    #[tokio::test]
    async fn start_and_stop_toggle_running_flag() {
        // ConnectionPool builds a rustls client; tests must pick a provider.
        let _ = rustls::crypto::ring::default_provider().install_default();

        // Bind a real ephemeral port so the listener can start.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let controller = ProxyController::new(
            addr,
            Arc::new(RequestCapture::new(100)),
            Arc::new(RuleEngine::new()),
            Arc::new(ScopeManager::new()),
            None,
            None,
            None,
            Arc::new(InterceptQueue::new()),
        );

        let started = controller.start().await.expect("start");
        assert!(started.running);
        assert_eq!(started.port, addr.port());

        let stopped = controller.stop().await.expect("stop");
        assert!(!stopped.running);
    }
}
