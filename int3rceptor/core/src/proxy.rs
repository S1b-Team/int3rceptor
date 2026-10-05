use crate::capture::{CapturedRequest, CapturedResponse, RequestCapture};
use crate::connection_pool::{ConnectionPool, ProxyBody};
use crate::error::{ProxyError, Result};
use crate::intercept::{HeldRequest, InterceptDecision, InterceptQueue};
use crate::metrics::metrics;
use crate::rules::RuleEngine;
use crate::scanner::Scanner;
use crate::scope::ScopeManager;
use crate::tls::TlsInterceptor;
use crate::websocket::WsCapture;
use crate::ws_proxy::{self, is_websocket_upgrade};
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::header::{HeaderMap, CONNECTION, HOST, UPGRADE};
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
use tokio::sync::oneshot;
use tracing::{debug, info, info_span, warn, Instrument};

/// Request ID counter for correlation
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);
static WS_CONN_COUNTER: AtomicU64 = AtomicU64::new(1);

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
    intercept: Arc<InterceptQueue>,
    ws_capture: Arc<WsCapture>,
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
            intercept: Arc::new(InterceptQueue::new()),
            ws_capture: Arc::new(WsCapture::new(10_000)),
        }
    }

    pub fn with_intercept(mut self, intercept: Arc<InterceptQueue>) -> Self {
        self.intercept = intercept;
        self
    }

    /// Record WebSocket upgrades in the shared capture used by the API.
    pub fn with_ws_capture(mut self, ws_capture: Arc<WsCapture>) -> Self {
        self.ws_capture = ws_capture;
        self
    }

    pub fn with_pool(mut self, pool: ConnectionPool) -> Self {
        self.pool = pool;
        self
    }

    pub async fn run(self) -> Result<()> {
        let (_tx, rx) = oneshot::channel();
        self.run_until(rx).await
    }

    pub async fn run_until(self, mut shutdown: oneshot::Receiver<()>) -> Result<()> {
        info!(addr = %self.addr, "Starting proxy server");
        let listener = TcpListener::bind(self.addr).await?;
        let capture = self.capture.clone();
        let pool = self.pool.clone();
        let rules = self.rules.clone();
        let scope = self.scope.clone();
        let tls = self.tls.clone();
        let plugins = self.plugins.clone();
        let scanner = self.scanner.clone();
        let intercept = self.intercept.clone();
        let ws_capture = self.ws_capture.clone();

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    info!(addr = %self.addr, "Proxy shutdown requested");
                    break;
                }
                accepted = listener.accept() => {
                    let (stream, peer) = accepted?;

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
                    let intercept = intercept.clone();
                    let ws_capture = ws_capture.clone();
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
                                let intercept = intercept.clone();
                                let ws_capture = ws_capture.clone();

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
                                            intercept.clone(),
                                            ws_capture.clone(),
                                            false,
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
                                                crate::telemetry::sentry::capture_anyhow(
                                                    &anyhow::anyhow!("{}", err),
                                                    "proxy",
                                                    &[("action", "handle_request")],
                                                );
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
                                .serve_connection_with_upgrades(io, service)
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

        Ok(())
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
    intercept: Arc<InterceptQueue>,
    ws_capture: Arc<WsCapture>,
    upstream_tls: bool,
) -> Result<Response<ProxyBody>> {
    if req.method() == Method::CONNECT {
        return handle_connect(
            req, capture, pool, rules, scope, tls, plugins, scanner, intercept, ws_capture,
        );
    }

    forward_request(
        req,
        pool,
        capture,
        rules,
        scope,
        plugins,
        scanner,
        intercept,
        ws_capture,
        upstream_tls,
    )
    .await
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

fn host_from_authority(authority: &str) -> String {
    if let Some(rest) = authority.strip_prefix('[') {
        return rest.split(']').next().unwrap_or(authority).to_string();
    }
    match authority.rsplit_once(':') {
        Some((host, port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => {
            host.to_string()
        }
        _ => authority.to_string(),
    }
}

/// Decrypted CONNECT traffic is plain HTTP on the inside, but the origin is HTTPS.
fn mark_intercepted_https(uri: Uri, upstream_tls: bool) -> Result<Uri> {
    if !upstream_tls || uri.scheme_str() == Some("https") {
        return Ok(uri);
    }
    let auth = uri
        .authority()
        .map(|value| value.to_string())
        .ok_or_else(|| ProxyError::InvalidRequest("missing host".into()))?;
    let path = uri
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());
    Ok(format!("https://{auth}{path}").parse()?)
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
    intercept: Arc<InterceptQueue>,
    ws_capture: Arc<WsCapture>,
    upstream_tls: bool,
) -> Result<Response<ProxyBody>> {
    if is_websocket_upgrade(req.headers()) {
        return forward_websocket(
            req,
            pool,
            capture,
            rules,
            scope,
            plugins,
            scanner,
            intercept,
            ws_capture,
            upstream_tls,
        )
        .await;
    }

    let mut target_uri =
        mark_intercepted_https(normalize_uri(req.uri(), req.headers())?, upstream_tls)?;

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

    // Interactive intercept: hold in-scope traffic when the queue is enabled.
    if intercept.is_enabled() {
        let held = HeldRequest {
            id: 0,
            method: record.method.clone(),
            url: record.url.clone(),
            headers: record.headers.clone(),
            body: body_bytes.clone(),
            tls,
            timestamp_ms: record.timestamp_ms,
        };
        match intercept.hold(held).await {
            InterceptDecision::Drop => {
                debug!(uri = %target_uri, "Request dropped by intercept queue");
                capture.push(record, None);
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(ProxyBody::from(Bytes::from_static(b"Dropped by intercept")))
                    .unwrap_or_else(|_| Response::new(ProxyBody::from(Bytes::new()))));
            }
            InterceptDecision::Edit {
                method,
                url,
                headers,
                body,
            } => {
                if let Some(method) = method {
                    if let Ok(parsed) = method.parse::<Method>() {
                        parts.method = parsed;
                        record.method = method;
                    }
                }
                if let Some(url) = url {
                    if let Ok(parsed) = url.parse::<Uri>() {
                        parts.uri = parsed.clone();
                        target_uri = parsed;
                        record.url = url;
                    }
                }
                if let Some(headers) = headers {
                    parts.headers.clear();
                    for (name, value) in &headers {
                        if let (Ok(n), Ok(v)) = (
                            hyper::header::HeaderName::from_bytes(name.as_bytes()),
                            hyper::header::HeaderValue::from_str(value),
                        ) {
                            parts.headers.insert(n, v);
                        }
                    }
                    record.headers = headers;
                }
                if let Some(body) = body {
                    body_bytes = body;
                    record.body = body_bytes.clone();
                }
            }
            InterceptDecision::Forward => {}
        }
    }

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
    intercept: Arc<InterceptQueue>,
    ws_capture: Arc<WsCapture>,
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
        let intercept = intercept.clone();
        let ws_capture = ws_capture.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_tls_connect(
                req, authority, capture, pool, rules, scope, tls, plugins, scanner, intercept,
                ws_capture,
            )
            .await
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
    authority: String,
    capture: Arc<RequestCapture>,
    pool: ConnectionPool,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Arc<TlsInterceptor>,
    plugins: Option<Arc<crate::plugin::PluginManager>>,
    scanner: Option<Arc<Scanner>>,
    intercept: Arc<InterceptQueue>,
    ws_capture: Arc<WsCapture>,
) -> Result<()> {
    let upgraded = hyper::upgrade::on(req).await?;

    // Track TLS handshake
    let host = host_from_authority(&authority);
    let stream = match tls.acceptor_for(&host).accept(TokioIo::new(upgraded)).await {
        Ok(s) => {
            metrics().record_tls_handshake();
            debug!("TLS handshake completed");
            s
        }
        Err(e) => {
            metrics().record_tls_error();
            warn!(%e, "TLS handshake failed");
            let err = ProxyError::tls_handshake("unknown", e.to_string());
            crate::telemetry::sentry::capture_anyhow(
                &anyhow::anyhow!("{}", err),
                "proxy",
                &[("action", "tls_handshake")],
            );
            return Err(err);
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
        let intercept = intercept.clone();
        let ws_capture = ws_capture.clone();
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
                intercept,
                ws_capture,
                true,
            )
            .await
        }
    });

    AutoBuilder::new(TokioExecutor::new())
        .serve_connection_with_upgrades(stream, service)
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

#[allow(clippy::too_many_arguments)]
async fn forward_websocket(
    mut req: Request<Incoming>,
    pool: ConnectionPool,
    capture: Arc<RequestCapture>,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    plugins: Option<Arc<crate::plugin::PluginManager>>,
    scanner: Option<Arc<Scanner>>,
    intercept: Arc<InterceptQueue>,
    ws_capture: Arc<WsCapture>,
    upstream_tls: bool,
) -> Result<Response<ProxyBody>> {
    let client_upgrade = hyper::upgrade::on(&mut req);
    let mut target_uri =
        mark_intercepted_https(normalize_uri(req.uri(), req.headers())?, upstream_tls)?;

    if let Some(host) = target_uri.host() {
        metrics().record_host_request(host);
    }

    let in_scope = scope.is_in_scope(&target_uri.to_string());

    if in_scope {
        if let Some(ref plugin_manager) = plugins {
            use crate::plugin::hooks::{HookContext, PluginHook};
            let hook_ctx = HookContext::new()
                .with_method(req.method().as_str())
                .with_url(target_uri.to_string());
            let _ = plugin_manager.execute_hook(PluginHook::OnRequest, hook_ctx);
        }
    }

    let (mut parts, body) = req.into_parts();
    parts.uri = target_uri.clone();
    let mut body_bytes = body.collect().await?.to_bytes().to_vec();
    metrics().record_bytes_received(body_bytes.len() as u64);

    let tls = target_uri.scheme_str() == Some("https");
    if in_scope {
        let rules_count = rules.get_rules().len();
        rules.apply_request_rules(&mut parts, &mut body_bytes);
        if rules_count > 0 {
            metrics().record_rules_applied(rules_count as u64);
        }
    }

    let mut record = CapturedRequest::new(parts.method.to_string(), target_uri.to_string(), tls);
    record.headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();
    record.body = body_bytes.clone();

    if in_scope && intercept.is_enabled() {
        let held = HeldRequest {
            id: 0,
            method: record.method.clone(),
            url: record.url.clone(),
            headers: record.headers.clone(),
            body: body_bytes.clone(),
            tls,
            timestamp_ms: record.timestamp_ms,
        };
        match intercept.hold(held).await {
            InterceptDecision::Drop => {
                capture.push(record, None);
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(ProxyBody::from(Bytes::from_static(b"Dropped by intercept")))
                    .unwrap_or_else(|_| Response::new(ProxyBody::from(Bytes::new()))));
            }
            InterceptDecision::Edit {
                method,
                url,
                headers,
                body,
            } => {
                if let Some(method) = method {
                    if let Ok(parsed) = method.parse::<Method>() {
                        parts.method = parsed;
                        record.method = method;
                    }
                }
                if let Some(url) = url {
                    if let Ok(parsed) = url.parse::<Uri>() {
                        parts.uri = parsed.clone();
                        target_uri = parsed;
                        record.url = url;
                    }
                }
                if let Some(headers) = headers {
                    parts.headers.clear();
                    for (name, value) in &headers {
                        if let (Ok(n), Ok(v)) = (
                            hyper::header::HeaderName::from_bytes(name.as_bytes()),
                            hyper::header::HeaderValue::from_str(value),
                        ) {
                            parts.headers.insert(n, v);
                        }
                    }
                    record.headers = headers;
                }
                if let Some(body) = body {
                    body_bytes = body;
                    record.body = body_bytes.clone();
                }
            }
            InterceptDecision::Forward => {}
        }
    }

    let mut outbound = Request::builder()
        .method(parts.method.clone())
        .uri(target_uri.clone())
        .version(http::Version::HTTP_11)
        .body(ProxyBody::from(Bytes::from(body_bytes)))
        .map_err(|err| ProxyError::InvalidRequest(err.to_string()))?;
    *outbound.headers_mut() = parts.headers.clone();

    let client = pool.http1_client();
    let start = Instant::now();
    let mut response = client.request(outbound).await?;
    let duration = start.elapsed();

    if response.status() != StatusCode::SWITCHING_PROTOCOLS {
        let (mut resp_parts, body) = response.into_parts();
        let mut resp_body = body.collect().await?.to_bytes().to_vec();
        metrics().record_bytes_sent(resp_body.len() as u64);
        if in_scope {
            rules.apply_response_rules(&mut resp_parts, &mut resp_body);
            let captured_response = CapturedResponse {
                request_id: 0,
                status_code: resp_parts.status.as_u16(),
                headers: resp_parts
                    .headers
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
                    .collect(),
                body: resp_body.clone(),
                duration_ms: duration.as_millis(),
            };
            let entry = crate::capture::CaptureEntry {
                request: record.clone(),
                response: Some(captured_response.clone()),
            };
            if let Some(scanner) = scanner {
                scanner.passive_scan(&entry);
            }
            capture.push(record, Some(captured_response));
        }
        return Ok(Response::from_parts(
            resp_parts,
            ProxyBody::from(Bytes::from(resp_body)),
        ));
    }

    let server_upgrade = hyper::upgrade::on(&mut response);
    let (mut resp_parts, resp_body) = response.into_parts();
    resp_parts.extensions.clear();
    if resp_parts.headers.get(UPGRADE).is_none() {
        resp_parts.headers.insert(
            UPGRADE,
            hyper::header::HeaderValue::from_static("websocket"),
        );
    }
    if resp_parts.headers.get(CONNECTION).is_none() {
        resp_parts.headers.insert(
            CONNECTION,
            hyper::header::HeaderValue::from_static("Upgrade"),
        );
    }

    if in_scope {
        let captured_response = CapturedResponse {
            request_id: 0,
            status_code: resp_parts.status.as_u16(),
            headers: resp_parts
                .headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
                .collect(),
            body: Vec::new(),
            duration_ms: duration.as_millis(),
        };
        let entry = crate::capture::CaptureEntry {
            request: record.clone(),
            response: Some(captured_response.clone()),
        };
        if let Some(scanner) = scanner {
            scanner.passive_scan(&entry);
        }
        capture.push(record, Some(captured_response));

        let id = format!("ws-{}", WS_CONN_COUNTER.fetch_add(1, Ordering::Relaxed));
        ws_capture.register_connection(id.clone(), ws_proxy::display_url(&target_uri));
        let ws_for_task = ws_capture.clone();
        let id_task = id.clone();
        tokio::spawn(async move {
            let _ = resp_body.collect().await;
            let upgraded = tokio::join!(client_upgrade, server_upgrade);
            match upgraded {
                (Ok(client), Ok(server)) => {
                    metrics().websocket_opened();
                    ws_proxy::relay(client, server, Some(ws_for_task.clone()), id_task.clone())
                        .await;
                    metrics().websocket_closed();
                    ws_for_task.close_connection(&id_task);
                }
                _ => {
                    ws_for_task.close_connection(&id_task);
                }
            }
        });
    } else {
        tokio::spawn(async move {
            let _ = resp_body.collect().await;
            if let (Ok(client), Ok(server)) = tokio::join!(client_upgrade, server_upgrade) {
                ws_proxy::relay(client, server, None, String::new()).await;
            }
        });
    }

    debug!(uri = %target_uri, "WebSocket upgrade forwarded");
    Ok(Response::from_parts(
        resp_parts,
        ProxyBody::from(Bytes::new()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::RuleEngine;
    use crate::scope::ScopeManager;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn masked_text(payload: &[u8]) -> Vec<u8> {
        let mask = [0x11, 0x22, 0x33, 0x44];
        let mut frame = vec![0x81, 0x80 | (payload.len() as u8)];
        frame.extend_from_slice(&mask);
        for (i, byte) in payload.iter().enumerate() {
            frame.push(byte ^ mask[i % 4]);
        }
        frame
    }

    async fn read_headers(sock: &mut tokio::net::TcpStream) -> String {
        let mut buf = Vec::new();
        let mut tmp = [0u8; 1024];
        loop {
            let n = tokio::time::timeout(Duration::from_secs(3), sock.read(&mut tmp))
                .await
                .expect("header read timeout")
                .expect("header read");
            assert!(n > 0, "socket closed before headers");
            buf.extend_from_slice(&tmp[..n]);
            if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }
        String::from_utf8_lossy(&buf).into_owned()
    }

    async fn echo_server(listener: TcpListener) {
        let (mut sock, _) = listener.accept().await.expect("upstream accept");
        let headers = read_headers(&mut sock).await;
        assert!(
            headers.to_ascii_lowercase().contains("upgrade: websocket"),
            "upstream did not see a websocket upgrade: {headers}"
        );
        let response = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
        sock.write_all(response).await.expect("101 write");
        let frame = ws_proxy::read_frame(&mut sock)
            .await
            .expect("upstream frame");
        let mut echo = vec![0x81, frame.payload.len() as u8];
        echo.extend_from_slice(&frame.payload);
        sock.write_all(&echo).await.expect("echo write");
        let mut tmp = [0u8; 16];
        let _ = sock.read(&mut tmp).await;
    }

    #[tokio::test]
    async fn proxy_records_websocket_upgrade_and_frame() {
        let _ = rustls::crypto::ring::default_provider().install_default();

        let upstream = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("upstream bind");
        let upstream_addr = upstream.local_addr().unwrap();
        tokio::spawn(echo_server(upstream));

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("proxy bind");
        let proxy_addr = listener.local_addr().unwrap();
        drop(listener);

        let ws_capture = Arc::new(WsCapture::new(100));
        let proxy = ProxyServer::new(
            proxy_addr,
            Arc::new(RequestCapture::new(100)),
            Arc::new(RuleEngine::new()),
            Arc::new(ScopeManager::new()),
            None,
            None,
            None,
        )
        .with_ws_capture(ws_capture.clone());
        tokio::spawn(async move {
            if let Err(err) = proxy.run().await {
                panic!("proxy exited: {err}");
            }
        });

        let mut client = None;
        for _ in 0..50 {
            match tokio::net::TcpStream::connect(proxy_addr).await {
                Ok(sock) => {
                    client = Some(sock);
                    break;
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
            }
        }
        let mut client = client.expect("proxy did not accept");

        let request = format!(
            "GET http://{upstream_addr}/ws HTTP/1.1\r\nHost: {upstream_addr}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        client
            .write_all(request.as_bytes())
            .await
            .expect("client write");
        let response = read_headers(&mut client).await;
        assert!(
            response.starts_with("HTTP/1.1 101"),
            "proxy did not return 101: {response}"
        );

        let payload = b"hello-ws";
        client
            .write_all(&masked_text(payload))
            .await
            .expect("frame write");
        let echoed =
            tokio::time::timeout(Duration::from_secs(3), ws_proxy::read_frame(&mut client))
                .await
                .expect("echo timeout")
                .expect("echo frame");
        assert_eq!(echoed.payload, payload);

        let connections = ws_capture.get_connections();
        assert_eq!(
            connections.len(),
            1,
            "proxy never recorded the websocket upgrade"
        );
        let frames = ws_capture.get_frames(&connections[0].id);
        assert!(
            frames.iter().any(|frame| frame.payload == payload),
            "proxy never recorded the websocket frame: {frames:?}"
        );
    }
}
