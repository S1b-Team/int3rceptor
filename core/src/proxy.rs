use crate::capture::{CapturedRequest, CapturedResponse, RequestCapture};
use crate::connection_pool::{ConnectionPool, ProxyBody};
use crate::error::{ProxyError, Result};
use crate::rules::RuleEngine;
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
use std::sync::Arc;
use std::time::Instant;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

#[derive(Clone)]
pub struct ProxyServer {
    addr: SocketAddr,
    capture: Arc<RequestCapture>,
    pool: ConnectionPool,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Option<Arc<TlsInterceptor>>,
}

impl ProxyServer {
    pub fn new(
        addr: SocketAddr,
        capture: Arc<RequestCapture>,
        rules: Arc<RuleEngine>,
        scope: Arc<ScopeManager>,
        tls: Option<Arc<TlsInterceptor>>,
    ) -> Self {
        Self {
            addr,
            capture,
            pool: ConnectionPool::new(),
            rules,
            scope,
            tls,
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting proxy on {}", self.addr);
        let listener = TcpListener::bind(self.addr).await?;
        let capture = self.capture.clone();
        let pool = self.pool.clone();
        let rules = self.rules.clone();
        let scope = self.scope.clone();
        let tls = self.tls.clone();
        loop {
            let (stream, peer) = listener.accept().await?;
            let capture = capture.clone();
            let pool = pool.clone();
            let rules = rules.clone();
            let scope = scope.clone();
            let tls = tls.clone();
            let peer_addr = peer;
            tokio::spawn(async move {
                let service = service_fn(move |req: Request<Incoming>| {
                    let capture = capture.clone();
                    let pool = pool.clone();
                    let rules = rules.clone();
                    let scope = scope.clone();
                    let tls = tls.clone();
                    async move {
                        match handle_request(
                            req,
                            pool.clone(),
                            capture.clone(),
                            rules.clone(),
                            scope.clone(),
                            tls.clone(),
                        )
                        .await
                        {
                            Ok(res) => Ok::<_, hyper::Error>(res),
                            Err(err) => {
                                warn!(%err, "proxy error");
                                Ok(error_response(err))
                            }
                        }
                    }
                });

                let io = TokioIo::new(stream);
                if let Err(err) = AutoBuilder::new(TokioExecutor::new())
                    .serve_connection(io, service)
                    .await
                {
                    warn!(%err, peer = %peer_addr, "connection error");
                }
            });
        }
    }
}

async fn handle_request(
    req: Request<Incoming>,
    pool: ConnectionPool,
    capture: Arc<RequestCapture>,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Option<Arc<TlsInterceptor>>,
) -> Result<Response<ProxyBody>> {
    if req.method() == Method::CONNECT {
        return handle_connect(req, capture, pool, rules, scope, tls);
    }

    forward_request(req, pool, capture, rules, scope).await
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
        .ok_or_else(|| ProxyError::Other("missing host".into()))?;

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
) -> Result<Response<ProxyBody>> {
    let target_uri = normalize_uri(req.uri(), req.headers())?;

    // Check Scope
    if !scope.is_in_scope(&target_uri.to_string()) {
        // Forward without capturing
        let (parts, body) = req.into_parts();
        let mut parts = parts;
        parts.uri = target_uri;
        let body_bytes = body.collect().await?.to_bytes();
        let forward_req = Request::from_parts(parts, ProxyBody::from(body_bytes));
        let client = pool.client();
        let response = client.request(forward_req).await?;
        let (parts, body) = response.into_parts();
        let body_bytes = body.collect().await?.to_bytes();
        return Ok(Response::from_parts(parts, ProxyBody::from(body_bytes)));
    }
    let tls = target_uri.scheme_str() == Some("https");

    let (mut parts, body) = req.into_parts();
    parts.uri = target_uri.clone();

    let mut body_bytes = body.collect().await?.to_bytes().to_vec();

    // Apply Request Rules
    rules.apply_request_rules(&mut parts, &mut body_bytes);

    let mut record = CapturedRequest::new(parts.method.to_string(), target_uri.to_string(), tls);
    record.headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();
    record.body = body_bytes.clone();

    let forward_req = Request::from_parts(parts, ProxyBody::from(Bytes::from(body_bytes.clone())));
    let client = pool.client();
    let start = Instant::now();
    let response = client.request(forward_req).await?;
    let duration = start.elapsed().as_millis();

    let (mut parts, body) = response.into_parts();
    let mut body_bytes = body.collect().await?.to_bytes().to_vec();

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
        duration_ms: duration,
    };
    capture.push(record, Some(captured_response));

    Ok(Response::from_parts(
        parts,
        ProxyBody::from(Bytes::from(body_bytes)),
    ))
}

fn handle_connect(
    req: Request<Incoming>,
    capture: Arc<RequestCapture>,
    pool: ConnectionPool,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Option<Arc<TlsInterceptor>>,
) -> Result<Response<ProxyBody>> {
    let authority = req
        .uri()
        .authority()
        .map(|a| a.to_string())
        .ok_or_else(|| ProxyError::Other("CONNECT missing authority".into()))?;

    let record = CapturedRequest::new("CONNECT", format!("https://{authority}"), true);
    capture.push(record, None);

    if let Some(tls) = tls {
        let capture = capture.clone();
        let rules = rules.clone();
        let scope = scope.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_tls_connect(req, capture, pool, rules, scope, tls).await {
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

async fn handle_tls_connect(
    req: Request<Incoming>,
    capture: Arc<RequestCapture>,
    pool: ConnectionPool,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Arc<TlsInterceptor>,
) -> Result<()> {
    let upgraded = hyper::upgrade::on(req).await?;
    let stream = tls.acceptor.accept(TokioIo::new(upgraded)).await?;
    let stream = TokioIo::new(stream);
    let service = service_fn(move |req: Request<Incoming>| {
        let capture = capture.clone();
        let pool = pool.clone();
        let rules = rules.clone();
        let scope = scope.clone();
        let tls = Some(tls.clone());
        async move {
            handle_request(
                req,
                pool.clone(),
                capture.clone(),
                rules.clone(),
                scope.clone(),
                tls,
            )
            .await
        }
    });

    AutoBuilder::new(TokioExecutor::new())
        .serve_connection(stream, service)
        .await
        .map_err(|e| ProxyError::Other(e.to_string()))?;
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
