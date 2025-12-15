use crate::capture::{CapturedRequest, CapturedResponse, RequestCapture};
use crate::connection_pool::{ConnectionPool, ProxyBody};
use crate::error::{ProxyError, Result};
use crate::metrics::metrics;
use crate::rules::RuleEngine;
use crate::scanner::Scanner;
use crate::scope::ScopeManager;
use crate::tls::TlsInterceptor;
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::header::{HeaderMap, HOST};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode, Uri};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, info_span, warn, Instrument};

/// Request ID counter for correlation
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

// ...

#[derive(Clone)]
pub struct ProxyServer {
    addr: SocketAddr,
    capture: Arc<RequestCapture>,
    pool: ConnectionPool,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Option<Arc<TlsInterceptor>>,
    plugins: Option<Arc<crate::plugin::PluginManager>>,
    scanner: Option<Arc<Scanner>>,
}

impl ProxyServer {
    pub fn new(
        addr: SocketAddr,
        capture: Arc<RequestCapture>,
        rules: Arc<RuleEngine>,
        scope: Arc<ScopeManager>,
        tls: Option<Arc<TlsInterceptor>>,
        plugins: Option<Arc<crate::plugin::PluginManager>>,
        scanner: Option<Arc<Scanner>>,
    ) -> Self {
        Self {
            addr,
            capture,
            pool: ConnectionPool::new(),
            rules,
            scope,
            tls,
            plugins,
            scanner,
        }
    }

    pub async fn run(self) -> Result<()> {
        info!(addr = %self.addr, "Starting proxy server");
        let listener = TcpListener::bind(self.addr).await?;
        let capture = self.capture.clone();
        let pool = self.pool.clone();
        let rules = self.rules.clone();
        let scope = self.scope.clone();
        let tls = self.tls.clone();
        let plugins = self.plugins.clone();
        let scanner = self.scanner.clone();

        loop {
            let (stream, peer) = listener.accept().await?;

            // Track connection metrics
            metrics().connection_opened();
            debug!(peer = %peer, "Connection accepted");

            let capture = capture.clone();
            let pool = pool.clone();
            let rules = rules.clone();
            let scope = scope.clone();
            let tls = tls.clone();
            let plugins = plugins.clone();
            let scanner = scanner.clone();
            let peer_addr = peer;

            tokio::spawn(
                async move {
                    let service = service_fn(move |req: Request<Incoming>| {
                        let capture = capture.clone();
                        let pool = pool.clone();
                        let rules = rules.clone();
                        let scope = scope.clone();
                        let tls = tls.clone();
                        let plugins = plugins.clone();
                        let scanner = scanner.clone();

                        async move {
                            let request_id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
                            let method = req.method().to_string();
                            let uri = req.uri().to_string();

                            // Create span for request tracing
                            let span = info_span!(
                                "request",
                                id = request_id,
                                method = %method,
                                uri = %uri,
                            );

                            async {
                                metrics().record_request();

                                match handle_request(
                                    req,
                                    pool.clone(),
                                    capture.clone(),
                                    rules.clone(),
                                    scope.clone(),
                                    tls.clone(),
                                    plugins.clone(),
                                    scanner.clone(),
                                )
                                .await
                                {
                                    Ok(res) => {
                                        metrics().record_request_success();
                                        metrics().record_response(res.status().as_u16());
                                        debug!(status = %res.status(), "Request completed");
                                        Ok::<_, hyper::Error>(res)
                                    }
                                    Err(err) => {
                                        metrics().record_request_error();
                                        warn!(%err, "Proxy error");
                                        Ok(error_response(err))
                                    }
                                }
                            }
                            .instrument(span)
                            .await
                        }
                    });

                    let io = TokioIo::new(stream);
                    if let Err(err) = AutoBuilder::new(TokioExecutor::new())
                        .serve_connection(io, service)
                        .await
                    {
                        warn!(%err, peer = %peer_addr, "Connection error");
                    }

                    // Track connection close
                    metrics().connection_closed();
                    debug!(peer = %peer_addr, "Connection closed");
                }
                .instrument(info_span!("connection", peer = %peer_addr)),
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_request(
    req: Request<Incoming>,
    pool: ConnectionPool,
    capture: Arc<RequestCapture>,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Option<Arc<TlsInterceptor>>,
    plugins: Option<Arc<crate::plugin::PluginManager>>,
    scanner: Option<Arc<Scanner>>,
) -> Result<Response<ProxyBody>> {
    if req.method() == Method::CONNECT {
        return handle_connect(req, capture, pool, rules, scope, tls, plugins, scanner);
    }

    forward_request(req, pool, capture, rules, scope, plugins, scanner).await
}

fn error_response(err: ProxyError) -> Response<ProxyBody> {
    Response::builder()
        .status(StatusCode::BAD_GATEWAY)
        .body(ProxyBody::from(Bytes::from(format!("Proxy error: {err}"))))
        .unwrap_or_else(|_| Response::new(ProxyBody::from(Bytes::new())))
}

fn host_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

fn normalize_uri(uri: &Uri, headers: &HeaderMap) -> Result<Uri> {
    if uri.scheme().is_some() && uri.authority().is_some() {
        return Ok(uri.clone());
    }

    let authority = uri
        .authority()
        .map(|a| a.to_string())
        .or_else(|| host_from_headers(headers))
        .ok_or_else(|| ProxyError::InvalidRequest("missing host header".into()))?;

    let path = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
    let full = format!("http://{authority}{path}");
    Ok(full.parse()?)
}

async fn forward_request(
    req: Request<Incoming>,
    pool: ConnectionPool,
    capture: Arc<RequestCapture>,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    plugins: Option<Arc<crate::plugin::PluginManager>>,
    scanner: Option<Arc<Scanner>>,
) -> Result<Response<ProxyBody>> {
    let target_uri = normalize_uri(req.uri(), req.headers())?;

    // Track host metrics
    if let Some(host) = target_uri.host() {
        metrics().record_host_request(host);
    }

    // Execute on_request plugin hook
    if let Some(ref plugin_manager) = plugins {
        use crate::plugin::hooks::{HookContext, PluginHook};
        let hook_ctx = HookContext::new()
            .with_method(req.method().as_str())
            .with_url(target_uri.to_string());
        let _ = plugin_manager.execute_hook(PluginHook::OnRequest, hook_ctx);
    }

    // Check Scope
    if !scope.is_in_scope(&target_uri.to_string()) {
        debug!(uri = %target_uri, "Request out of scope, forwarding without capture");
        // Forward without capturing
        let (parts, body) = req.into_parts();
        let mut parts = parts;
        parts.uri = target_uri;
        let body_bytes = body.collect().await?.to_bytes();
        metrics().record_bytes_received(body_bytes.len() as u64);

        let forward_req = Request::from_parts(parts, ProxyBody::from(body_bytes));
        let client = pool.client();
        let response = client.request(forward_req).await?;
        let (parts, body) = response.into_parts();
        let body_bytes = body.collect().await?.to_bytes();
        metrics().record_bytes_sent(body_bytes.len() as u64);

        return Ok(Response::from_parts(parts, ProxyBody::from(body_bytes)));
    }

    let tls = target_uri.scheme_str() == Some("https");

    let (mut parts, body) = req.into_parts();
    parts.uri = target_uri.clone();

    let mut body_bytes = body.collect().await?.to_bytes().to_vec();
    metrics().record_bytes_received(body_bytes.len() as u64);

    // Apply Request Rules
    let rules_count = rules.get_rules().len();
    rules.apply_request_rules(&mut parts, &mut body_bytes);
    if rules_count > 0 {
        metrics().record_rules_applied(rules_count as u64);
    }

    let mut record = CapturedRequest::new(parts.method.to_string(), target_uri.to_string(), tls);
    record.headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();
    record.body = body_bytes.clone();

    let forward_req = Request::from_parts(parts, ProxyBody::from(Bytes::from(body_bytes.clone())));
    let client = pool.client();

    // Time the request
    let _timer = metrics().time_request();
    let start = Instant::now();
    let response = client.request(forward_req).await?;
    let duration = start.elapsed();

    let (mut parts, body) = response.into_parts();
    let mut body_bytes = body.collect().await?.to_bytes().to_vec();
    metrics().record_bytes_sent(body_bytes.len() as u64);

    // Execute on_response plugin hook
    if let Some(ref plugin_manager) = plugins {
        use crate::plugin::hooks::{HookContext, PluginHook};
        let hook_ctx = HookContext::new().with_status_code(parts.status.as_u16());
        let _ = plugin_manager.execute_hook(PluginHook::OnResponse, hook_ctx);
    }

    // Apply Response Rules
    rules.apply_response_rules(&mut parts, &mut body_bytes);

    let captured_response = CapturedResponse {
        request_id: 0,
        status_code: parts.status.as_u16(),
        headers: parts
            .headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect(),
        body: body_bytes.clone(),
        duration_ms: duration.as_millis(),
    };

    debug!(
        status = parts.status.as_u16(),
        duration_ms = duration.as_millis(),
        body_size = body_bytes.len(),
        "Request forwarded"
    );

    // Capture and Scan
    let entry = crate::capture::CaptureEntry {
        request: record.clone(),
        response: Some(captured_response.clone()),
    };

    // Passive Scan
    if let Some(scanner) = scanner {
        scanner.passive_scan(&entry);
    }

    capture.push(record, Some(captured_response));

    Ok(Response::from_parts(
        parts,
        ProxyBody::from(Bytes::from(body_bytes)),
    ))
}

#[allow(clippy::too_many_arguments)]
fn handle_connect(
    req: Request<Incoming>,
    capture: Arc<RequestCapture>,
    pool: ConnectionPool,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Option<Arc<TlsInterceptor>>,
    plugins: Option<Arc<crate::plugin::PluginManager>>,
    scanner: Option<Arc<Scanner>>,
) -> Result<Response<ProxyBody>> {
    let authority = req
        .uri()
        .authority()
        .map(|a| a.to_string())
        .ok_or_else(|| ProxyError::InvalidRequest("CONNECT missing authority".into()))?;

    let record = CapturedRequest::new("CONNECT", format!("https://{authority}"), true);
    capture.push(record, None);

    if let Some(tls) = tls {
        let capture = capture.clone();
        let rules = rules.clone();
        let scope = scope.clone();
        let scanner = scanner.clone();
        tokio::spawn(async move {
            if let Err(err) =
                handle_tls_connect(req, capture, pool, rules, scope, tls, plugins, scanner).await
            {
                warn!(%err, "tls intercept error");
            }
        });
    } else {
        tokio::spawn(async move {
            if let Err(err) = tunnel(authority, req).await {
                warn!(%err, "connect tunnel error");
            }
        });
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(ProxyBody::from(Bytes::new()))
        .unwrap())
}

#[allow(clippy::too_many_arguments)]
async fn handle_tls_connect(
    req: Request<Incoming>,
    capture: Arc<RequestCapture>,
    pool: ConnectionPool,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Arc<TlsInterceptor>,
    plugins: Option<Arc<crate::plugin::PluginManager>>,
    scanner: Option<Arc<Scanner>>,
) -> Result<()> {
    let upgraded = hyper::upgrade::on(req).await?;

    // Track TLS handshake
    let stream = match tls.acceptor.accept(TokioIo::new(upgraded)).await {
        Ok(s) => {
            metrics().record_tls_handshake();
            debug!("TLS handshake completed");
            s
        }
        Err(e) => {
            metrics().record_tls_error();
            warn!(%e, "TLS handshake failed");
            return Err(ProxyError::tls_handshake("unknown", e.to_string()));
        }
    };
    let stream = TokioIo::new(stream);
    let service = service_fn(move |req: Request<Incoming>| {
        let capture = capture.clone();
        let pool = pool.clone();
        let rules = rules.clone();
        let scope = scope.clone();
        let tls = Some(tls.clone());
        let plugins = plugins.clone();
        let scanner = scanner.clone();
        async move {
            handle_request(
                req,
                pool.clone(),
                capture.clone(),
                rules.clone(),
                scope.clone(),
                tls,
                plugins,
                scanner,
            )
            .await
        }
    });

    AutoBuilder::new(TokioExecutor::new())
        .serve_connection(stream, service)
        .await
        .map_err(|e| ProxyError::internal(e.to_string()))?;
    Ok(())
}

async fn tunnel(host: String, req: Request<Incoming>) -> Result<()> {
    let upgraded = hyper::upgrade::on(req).await?;
    let addr = resolve_addr(&host);
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);
    copy_bidirectional(&mut upgraded, &mut server).await?;
    Ok(())
}

fn resolve_addr(host: &str) -> String {
    if host.contains(':') {
        host.to_string()
    } else {
        format!("{host}:443")
    }
}
