use crate::{
    models::{
        ActivityQuery, AppSettings, DashboardActivity, HeaderPatch, ManualRequest, ManualResponse,
        PluginInfo, PluginToggle, RepeatRequest,
    },
    state::AppState,
};
use axum::extract::{Extension, Multipart, Path, Query};
use axum::http::{header, HeaderMap, HeaderName, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::{delete, get, post};
use axum::Router;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{Method, Request, Uri};
use interceptor_core::capture::{CaptureEntry, CaptureQuery, CapturedRequest, CapturedResponse};
use interceptor_core::comparer::{CompareRequest, Comparer};
use interceptor_core::connection_pool::ProxyBody;
use interceptor_core::encoding::{Encoder, TransformRequest};
use interceptor_core::metrics;
use interceptor_core::plugin::config::PluginConfig;
use interceptor_core::rules::Rule;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

// Maximum allowed body size to prevent DoS attacks (10MB)
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

// ... existing handlers ...

async fn upload_plugin(
    Extension(state): Extension<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = if let Some(name) = field.file_name() {
            name.to_string()
        } else {
            continue;
        };

        if !file_name.ends_with(".wasm") {
            continue;
        }

        let data = if let Ok(bytes) = field.bytes().await {
            bytes
        } else {
            continue;
        };

        // Ensure plugins directory exists
        let plugins_dir = PathBuf::from("plugins");
        if !plugins_dir.exists() {
            let _ = fs::create_dir_all(&plugins_dir);
        }

        let file_path = plugins_dir.join(&file_name);
        if let Err(e) = fs::write(&file_path, &data) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to save file: {}", e) })),
            );
        }

        // Create config and load
        let plugin_name = file_name.trim_end_matches(".wasm").to_string();
        let config = PluginConfig {
            name: plugin_name.clone(),
            path: file_path,
            enabled: true,
            priority: 100,
            ..Default::default()
        };

        if let Err(e) = state.plugin_manager.load_plugin(config) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load plugin: {}", e) })),
            );
        }

        return (
            StatusCode::OK,
            Json(
                json!({ "message": "Plugin uploaded and loaded successfully", "name": plugin_name }),
            ),
        );
    }

    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "No .wasm file found in request" })),
    )
}

pub fn router() -> Router {
    Router::new()
        .route("/api/requests", get(list_requests).delete(clear_requests))
        .route("/api/requests/:id", get(get_request))
        .route("/api/requests/:id/repeat", post(repeat_request))
        .route("/api/repeater/send", post(send_manual_request))
        .route("/api/settings", get(get_settings).put(update_settings))
        .route("/api/plugins", get(list_plugins))
        .route("/api/plugins/upload", post(upload_plugin))
        .route("/api/plugins/:name/toggle", post(toggle_plugin))
        .route("/api/requests/export", get(export_requests))
        .route("/api/ca-cert", get(download_ca_cert))
        .route(
            "/api/rules",
            get(list_rules).post(add_rule).delete(clear_rules),
        )
        .route("/api/scope", get(get_scope).put(set_scope))
        .route("/api/intruder/generate", post(intruder_generate))
        .route(
            "/api/intruder/results",
            get(intruder_results).delete(intruder_clear),
        )
        .route("/api/intruder/start", post(intruder_start))
        .route("/api/intruder/stop", post(intruder_stop))
        // Scanner routes
        .route(
            "/api/scanner/config",
            get(scanner_get_config).put(scanner_set_config),
        )
        .route("/api/scanner/rules", get(scanner_get_rules))
        .route("/api/scanner/start", post(scanner_start))
        .route("/api/scanner/stop", post(scanner_stop))
        .route(
            "/api/scanner/findings",
            get(scanner_get_findings).delete(scanner_clear_findings),
        )
        .route("/api/scanner/stats", get(scanner_get_stats))
        // Encoding & Comparer routes
        .route("/api/encoding/transform", post(encoding_transform))
        .route("/api/comparer/diff", post(comparer_diff))
        .route("/api/websocket/connections", get(ws_connections))
        .route("/api/websocket/frames/:connection_id", get(ws_frames))
        .route("/api/websocket/clear", delete(ws_clear))
        // Project routes
        .route("/api/project/save", post(project_save))
        .route("/api/project/load", post(project_load))
        .route("/api/project/info", get(project_info).put(project_update))
        .route("/api/project/new", post(project_new))
        // Metrics and monitoring
        .route("/api/metrics", get(get_metrics))
        .route("/api/metrics/reset", post(reset_metrics))
        .route("/api/health", get(health_check))
        .route("/api/dashboard/activity", get(get_dashboard_activity))
        .route("/api/dashboard/activity/clear", delete(clear_dashboard_activity))
        // Security management routes
        .route(
            "/api/security/ip-filter",
            get(crate::security_routes::get_ip_filter_config)
                .put(crate::security_routes::set_ip_filter_config),
        )
        .route(
            "/api/security/ip-filter/allow",
            post(crate::security_routes::add_allowed_ip),
        )
        .route(
            "/api/security/ip-filter/block",
            post(crate::security_routes::add_blocked_ip),
        )
        .route(
            "/api/security/audit-log/info",
            get(crate::security_routes::get_audit_log_info),
        )
        .route(
            "/api/security/audit-log/rotate",
            post(crate::security_routes::rotate_audit_log),
        )
        // License routes
        .route("/api/license", get(get_license))
        // Proxy control
        .route("/api/proxy/status", get(proxy_status))
        .route("/api/proxy/start", post(proxy_start))
        .route("/api/proxy/stop", post(proxy_stop))
        // Interactive intercept queue
        .route("/api/intercept", get(list_intercept).put(set_intercept_enabled))
        .route("/api/intercept/:id/forward", post(intercept_forward))
        .route("/api/intercept/:id/drop", post(intercept_drop))
        .route("/api/intercept/:id/edit", post(intercept_edit))
}

async fn get_license(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let license = state.license_manager.license();
    let tier = state.license_manager.tier();

    Json(json!({
        "tier": tier,
        "license": license,
        "features": {
            "intruder": state.license_manager.check_feature("intruder"),
            "websocket": state.license_manager.check_feature("websocket"),
            "rules": state.license_manager.check_feature("rules"),
            "export": state.license_manager.check_feature("export"),
        }
    }))
}

async fn list_requests(
    Query(params): Query<ListParams>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let query: CaptureQuery = params.into();
    let items = state.capture.query(&query);
    Json(items)
}

async fn get_request(
    Path(id): Path<u64>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    match state.capture.get(id) {
        Some(item) => Json(item).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn repeat_request(
    Path(id): Path<u64>,
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<RepeatRequest>,
) -> impl IntoResponse {
    let Some(entry) = state.capture.get(id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let target_url = payload.url.as_deref().unwrap_or(&entry.request.url);
    let uri: Uri = match target_url.parse() {
        Ok(u) => u,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let body_bytes = if let Some(body) = payload.modified_body {
        let bytes = body.into_bytes();
        if bytes.len() > MAX_BODY_SIZE {
            return (StatusCode::PAYLOAD_TOO_LARGE, "Request body exceeds maximum size of 10MB").into_response();
        }
        bytes
    } else {
        entry.request.body.clone()
    };

    let method_str = payload.method.as_deref().unwrap_or(&entry.request.method);
    let method = match method_str.parse::<Method>() {
        Ok(m) => m,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let mut builder = Request::builder().method(method.clone()).uri(uri.clone());

    let header_source: Vec<HeaderPatch> = payload
        .headers
        .clone()
        .unwrap_or_else(|| to_header_patches(&entry.request.headers));

    for pair in &header_source {
        if pair.name.is_empty() {
            continue;
        }
        if let Ok(header_name) = HeaderName::from_bytes(pair.name.as_bytes()) {
            builder = builder.header(header_name, &pair.value);
        }
    }

    let request_body = ProxyBody::from(Bytes::from(body_bytes.clone()));
    let request = match builder.body(request_body) {
        Ok(req) => req,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let client = state.pool.client();
    let start = std::time::Instant::now();
    let response = match client.request(request).await {
        Ok(resp) => resp,
        Err(err) => {
            tracing::warn!(%err, "replay request failed");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };
    let duration = start.elapsed().as_millis();

    let (parts, body) = response.into_parts();
    let body_bytes_resp = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            tracing::warn!(%err, "failed to read replay response");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };
    let preview_len = body_bytes_resp.len().min(4096);
    let body_preview = BASE64.encode(&body_bytes_resp[..preview_len]);

    let mut captured_request = CapturedRequest::new(
        method.to_string(),
        uri.to_string(),
        uri.scheme_str() == Some("https"),
    );
    captured_request.headers = header_source
        .iter()
        .map(|h| (h.name.clone(), h.value.clone()))
        .collect();
    captured_request.body = body_bytes.clone();
    captured_request.timestamp_ms = OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;

    let response_headers: Vec<(String, String)> = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();

    let captured_response = CapturedResponse {
        request_id: 0,
        status_code: parts.status.as_u16(),
        headers: response_headers.clone(),
        body: body_bytes_resp.clone().to_vec(),
        duration_ms: duration,
    };
    state
        .capture
        .push(captured_request, Some(captured_response));

    Json(json!({
        "id": id,
        "status": parts.status.as_u16(),
        "duration_ms": duration,
        "timestamp_ms": (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000),
        "headers": response_headers,
        "body_preview": body_preview,
    }))
    .into_response()
}

async fn export_requests(
    Query(params): Query<ExportParams>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let query: CaptureQuery = params.filters.into();
    let entries = state.capture.query(&query);
    build_export_response(entries, params.format)
}

fn to_header_patches(headers: &[(String, String)]) -> Vec<HeaderPatch> {
    headers
        .iter()
        .map(|(name, value)| HeaderPatch {
            name: name.clone(),
            value: value.clone(),
        })
        .collect()
}

#[derive(Debug, Deserialize, Default)]
struct ListParams {
    method: Option<String>,
    host: Option<String>,
    status: Option<u16>,
    tls: Option<bool>,
    search: Option<String>,
    limit: Option<usize>,
}

impl From<ListParams> for CaptureQuery {
    fn from(value: ListParams) -> Self {
        CaptureQuery {
            method: value.method,
            host: value.host,
            status: value.status,
            tls: value.tls,
            search: value.search,
            limit: value.limit,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExportParams {
    #[serde(flatten)]
    filters: ListParams,
    #[serde(default)]
    format: ExportFormat,
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
enum ExportFormat {
    #[default]
    Json,
    Csv,
    Har,
}

fn build_export_response(entries: Vec<CaptureEntry>, format: ExportFormat) -> impl IntoResponse {
    match format {
        ExportFormat::Json => Json(entries).into_response(),
        ExportFormat::Csv => {
            let mut w = String::from("id,timestamp_ms,method,url,status,duration_ms\n");
            for entry in &entries {
                let status = entry
                    .response
                    .as_ref()
                    .map(|r| r.status_code.to_string())
                    .unwrap_or_else(|| "".into());
                let duration = entry
                    .response
                    .as_ref()
                    .map(|r| r.duration_ms.to_string())
                    .unwrap_or_else(|| "".into());
                w.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    entry.request.id,
                    entry.request.timestamp_ms,
                    entry.request.method,
                    entry.request.url.replace(',', " "),
                    status,
                    duration
                ));
            }
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, "text/csv".parse().unwrap());
            headers.insert(
                header::CONTENT_DISPOSITION,
                "attachment; filename=interceptor.csv".parse().unwrap(),
            );
            (headers, w).into_response()
        }
        ExportFormat::Har => {
            let log_entries: Vec<_> = entries
                .iter()
                .map(|entry| {
                    let started = OffsetDateTime::from_unix_timestamp_nanos(entry.request.timestamp_ms * 1_000_000)
                        .unwrap_or_else(|_| OffsetDateTime::now_utc());
                    let started_str = started
                        .format(&Rfc3339)
                        .unwrap_or_else(|_| started.to_string());
                    json!({
                        "startedDateTime": started_str,
                        "time": entry.response.as_ref().map(|r| r.duration_ms as u64).unwrap_or(0),
                        "request": {
                            "method": entry.request.method,
                            "url": entry.request.url,
                            "headers": entry.request.headers,
                            "bodySize": entry.request.body.len(),
                        },
                        "response": {
                            "status": entry.response.as_ref().map(|r| r.status_code).unwrap_or(0),
                            "headers": entry.response.as_ref().map(|r| r.headers.clone()).unwrap_or_default(),
                            "bodySize": entry.response.as_ref().map(|r| r.body.len()).unwrap_or(0),
                        }
                    })
                })
                .collect();
            let payload = json!({
                "log": {
                    "version": "1.2",
                    "creator": {"name": "Interceptor", "version": "0.1"},
                    "entries": log_entries,
                }
            });
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
            headers.insert(
                header::CONTENT_DISPOSITION,
                "attachment; filename=interceptor.har".parse().unwrap(),
            );
            (headers, Json(payload)).into_response()
        }
    }
}

async fn clear_requests(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let cleared_count = state.capture.len();
    state.capture.clear();
    Json(json!({ "cleared_count": cleared_count }))
}

async fn download_ca_cert(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    match state.cert_manager.ca_pem() {
        Ok(pem) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/x-pem-file".parse().unwrap(),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                "attachment; filename=interceptor-ca.pem".parse().unwrap(),
            );
            (headers, pem).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn list_rules(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let rules = state.rules.get_rules();
    Json(rules)
}

async fn add_rule(
    Extension(state): Extension<Arc<AppState>>,
    Json(rule): Json<Rule>,
) -> impl IntoResponse {
    state.rules.add_rule(rule);
    StatusCode::CREATED
}

async fn clear_rules(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    state.rules.clear_rules();
    StatusCode::NO_CONTENT
}

// Scope handlers
async fn get_scope(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<interceptor_core::scope::ScopeConfig> {
    Json(state.scope.get_config())
}

async fn set_scope(
    Extension(state): Extension<Arc<AppState>>,
    Json(config): Json<interceptor_core::scope::ScopeConfig>,
) -> StatusCode {
    state.scope.set_config(config);
    StatusCode::NO_CONTENT
}

// Intruder handlers
async fn intruder_generate(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<IntruderGenerateRequest>,
) -> impl IntoResponse {
    match state.intruder.generate_requests(&req.template, &req.config) {
        Ok(requests) => {
            // Only return request strings for preview, not payloads
            let request_strings: Vec<String> = requests.into_iter().map(|(r, _)| r).collect();
            Json(json!({ "requests": request_strings })).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

async fn intruder_results(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    Json(state.intruder.get_results())
}

async fn intruder_clear(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    state.intruder.clear_results();
    StatusCode::NO_CONTENT
}

async fn intruder_start(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<IntruderGenerateRequest>,
) -> impl IntoResponse {
    match state
        .intruder
        .start_attack(req.template, req.config, state.pool.clone())
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn intruder_stop(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    state.intruder.stop_attack();
    StatusCode::OK
}

#[derive(Deserialize)]
struct IntruderGenerateRequest {
    template: String,
    config: interceptor_core::intruder::IntruderConfig,
}

// WebSocket handlers
async fn ws_connections(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    Json(state.ws_capture.get_connections())
}

async fn ws_frames(
    Extension(state): Extension<Arc<AppState>>,
    Path(connection_id): Path<String>,
) -> impl IntoResponse {
    Json(state.ws_capture.get_frames(&connection_id))
}

async fn ws_clear(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    state.ws_capture.clear();
    StatusCode::NO_CONTENT
}

// Metrics handlers
async fn get_metrics() -> impl IntoResponse {
    let snapshot = metrics::metrics().snapshot();
    Json(snapshot)
}

async fn reset_metrics() -> impl IntoResponse {
    metrics::metrics().reset();
    StatusCode::NO_CONTENT
}

async fn health_check(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let m = metrics::metrics().snapshot();
    Json(json!({
        "status": "healthy",
        "uptime_secs": m.uptime_secs,
        "connections_active": m.connections_active,
        "requests_total": m.requests_total,
        "capture_count": state.capture.len(),
    }))
}

// Dashboard Activity handlers

async fn get_dashboard_activity(
    Query(params): Query<ActivityQuery>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let activities = state.capture.get_activity(&params);
    Json(activities)
}

async fn clear_dashboard_activity(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    state.capture.clear_activity();
    StatusCode::NO_CONTENT
}

/// Validates URL to prevent SSRF attacks
fn validate_url(url: &str) -> Result<reqwest::Url, String> {
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
    
    // Check scheme
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err("Only HTTP and HTTPS URLs are allowed".to_string());
    }
    
    // Check host
    let host = parsed.host_str().ok_or("URL must have a host")?;
    
    // Block private IP ranges and localhost
    if host == "localhost" 
        || host == "127.0.0.1"
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("172.")
        || host == "[::1]"
        || host.starts_with("[fc00:")
        || host.starts_with("[fe80:") {
        return Err("Access to internal addresses is not allowed".to_string());
    }
    
    // Check for IP addresses in private ranges
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        let is_internal = match ip {
            std::net::IpAddr::V4(ipv4) => {
                ipv4.is_loopback() || ipv4.is_private() || ipv4.is_link_local() || ipv4.is_multicast()
            }
            std::net::IpAddr::V6(ipv6) => {
                ipv6.is_loopback() || ipv6.is_multicast()
            }
        };
        if is_internal {
            return Err("Access to internal IP addresses is not allowed".to_string());
        }
    }
    
    Ok(parsed)
}

async fn send_manual_request(Json(payload): Json<ManualRequest>) -> impl IntoResponse {
    // Validate URL to prevent SSRF
    let parsed_url = match validate_url(&payload.url) {
        Ok(url) => url,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    // Only allow invalid certs in development mode
    let allow_invalid_certs = std::env::var("INTERCEPTOR_DEV_MODE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let client = Client::builder()
        .danger_accept_invalid_certs(allow_invalid_certs)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let method = match payload.method.parse::<reqwest::Method>() {
        Ok(m) => m,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let start = Instant::now();

    let mut req_builder = client.request(method, parsed_url);

    if let Some(headers) = payload.headers {
        for (k, v) in headers {
            req_builder = req_builder.header(k, v);
        }
    }

    if let Some(body) = payload.body {
        req_builder = req_builder.body(body);
    }

    match req_builder.send().await {
        Ok(res) => {
            let status = res.status().as_u16();
            let status_text = res.status().canonical_reason().unwrap_or("").to_string();

            let mut headers = std::collections::HashMap::new();
            for (k, v) in res.headers() {
                headers.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
            }

            let body_bytes = res.bytes().await.unwrap_or_default();
            let size_bytes = body_bytes.len();
            let body = String::from_utf8_lossy(&body_bytes).to_string();
            let time_ms = start.elapsed().as_millis() as u64;

            Json(ManualResponse {
                status,
                status_text,
                headers,
                body,
                time_ms,
                size_bytes,
            })
            .into_response()
        }
        Err(e) => (StatusCode::BAD_GATEWAY, format!("Request failed: {}", e)).into_response(),
    }
}

async fn get_settings(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let settings = state.settings.read().await;
    Json(settings.clone())
}

async fn update_settings(
    Extension(state): Extension<Arc<AppState>>,
    Json(new_settings): Json<AppSettings>,
) -> impl IntoResponse {
    let mut settings = state.settings.write().await;
    *settings = new_settings;
    Json(settings.clone())
}

async fn list_plugins(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let plugins = state.plugin_manager.list_plugins();
    let mut plugin_infos = Vec::new();

    for name in plugins {
        plugin_infos.push(PluginInfo {
            name: name.clone(),
            version: "1.0.0".to_string(),
            enabled: state.plugin_manager.is_loaded(&name),
            description: "WASM Plugin".to_string(),
        });
    }

    Json(plugin_infos)
}

async fn toggle_plugin(
    Path(name): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<PluginToggle>,
) -> impl IntoResponse {
    if payload.enabled {
        match state.plugin_manager.reload_plugin(&name) {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        match state.plugin_manager.unload_plugin(&name) {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}

// ============= SCANNER HANDLERS =============

async fn scanner_get_config(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    Json(state.scanner.get_config())
}

async fn scanner_set_config(
    Extension(state): Extension<Arc<AppState>>,
    Json(config): Json<interceptor_core::ScanConfig>,
) -> impl IntoResponse {
    state.scanner.configure(config);
    StatusCode::OK
}

async fn scanner_get_rules(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    Json(state.scanner.get_rules())
}

#[derive(Deserialize)]
struct ScanStartRequest {
    targets: Vec<String>,
}

async fn scanner_start(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<ScanStartRequest>,
) -> impl IntoResponse {
    match state
        .scanner
        .clone()
        .start_active_scan(payload.targets, state.pool.clone())
        .await
    {
        Ok(scan_id) => Json(json!({ "scan_id": scan_id, "status": "started" })).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn scanner_stop(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    state.scanner.stop_scan();
    Json(json!({ "status": "stopped" }))
}

async fn scanner_get_findings(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    Json(state.scanner.get_findings())
}

async fn scanner_clear_findings(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    state.scanner.clear_findings();
    StatusCode::OK
}

async fn scanner_get_stats(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    Json(state.scanner.get_stats())
}

// Encoding & Comparer Handlers

async fn encoding_transform(Json(req): Json<TransformRequest>) -> impl IntoResponse {
    let response = Encoder::transform(req);
    Json(response)
}

async fn comparer_diff(Json(req): Json<CompareRequest>) -> impl IntoResponse {
    let response = Comparer::compare(req);
    Json(response)
}

// Project Handlers

#[derive(Deserialize)]
struct ProjectSaveRequest {
    path: String,
    scope: Vec<String>,
}

#[derive(Deserialize)]
struct ProjectLoadRequest {
    path: String,
}

#[derive(Deserialize)]
struct ProjectNewRequest {
    name: String,
}

#[derive(Deserialize)]
struct ProjectUpdateRequest {
    name: Option<String>,
    description: Option<String>,
}

async fn project_save(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<ProjectSaveRequest>,
) -> impl IntoResponse {
    let settings = serde_json::to_value(&*state.settings.read().await).unwrap_or_default();
    match state.project_manager.export(req.scope, settings) {
        Ok(data) => match data.save_to_file(&req.path) {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn project_load(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<ProjectLoadRequest>,
) -> impl IntoResponse {
    match interceptor_core::project::ProjectData::load_from_file(&req.path) {
        Ok(data) => {
            // Restore scope
            state
                .scope
                .set_config(interceptor_core::scope::ScopeConfig {
                    includes: data.scope.clone(),
                    excludes: vec![], // TODO: Save excludes in project data
                });

            // Restore settings
            if let Ok(settings) = serde_json::from_value(data.settings.clone()) {
                *state.settings.write().await = settings;
            }

            match state.project_manager.import(data) {
                Ok(_) => StatusCode::OK.into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn project_info(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    Json(state.project_manager.current_info())
}

async fn project_update(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<ProjectUpdateRequest>,
) -> impl IntoResponse {
    state.project_manager.update_info(req.name, req.description);
    StatusCode::OK
}

async fn project_new(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<ProjectNewRequest>,
) -> impl IntoResponse {
    state.project_manager.new_project(req.name);
    // Clear other state
    state
        .scope
        .set_config(interceptor_core::scope::ScopeConfig::default());
    state.intruder.clear_results();
    state.ws_capture.clear();
    state.scanner.clear_findings();

    StatusCode::OK
}


// ============= PROXY CONTROL =============

async fn proxy_status(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    Json(state.proxy.status())
}

async fn proxy_start(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    // Keep listen address in sync with settings.
    let settings = state.settings.read().await;
    let addr = format!("{}:{}", settings.proxy.host, settings.proxy.port);
    drop(settings);
    if let Ok(parsed) = addr.parse() {
        state.proxy.set_addr(parsed);
    }

    match state.proxy.start().await {
        Ok(status) => Json(status).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err })),
        )
            .into_response(),
    }
}

async fn proxy_stop(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    match state.proxy.stop().await {
        Ok(status) => Json(status).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err })),
        )
            .into_response(),
    }
}

// ============= INTERCEPT QUEUE =============

#[derive(Deserialize)]
struct InterceptEnableRequest {
    enabled: bool,
}

#[derive(Deserialize)]
struct InterceptEditRequest {
    method: Option<String>,
    url: Option<String>,
    headers: Option<Vec<(String, String)>>,
    /// Raw body text (UTF-8). Prefer this for UI edits.
    body: Option<String>,
    /// Optional base64 body; used when body is omitted.
    body_base64: Option<String>,
}

fn held_to_json(held: &interceptor_core::HeldRequest) -> serde_json::Value {
    let body_text = String::from_utf8(held.body.clone()).ok();
    json!({
        "id": held.id,
        "method": held.method,
        "url": held.url,
        "headers": held.headers,
        "body_base64": BASE64.encode(&held.body),
        "body_text": body_text,
        "tls": held.tls,
        "timestamp_ms": held.timestamp_ms,
    })
}

async fn list_intercept(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let status = state.intercept.status();
    let items: Vec<_> = state.intercept.list().iter().map(held_to_json).collect();
    Json(json!({
        "enabled": status.enabled,
        "held_count": status.held_count,
        "items": items,
    }))
}

async fn set_intercept_enabled(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<InterceptEnableRequest>,
) -> impl IntoResponse {
    state.intercept.set_enabled(payload.enabled);
    let status = state.intercept.status();
    Json(json!({
        "enabled": status.enabled,
        "held_count": status.held_count,
    }))
}

async fn intercept_forward(
    Path(id): Path<u64>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    if state.intercept.forward(id) {
        Json(json!({ "id": id, "action": "forward" })).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn intercept_drop(
    Path(id): Path<u64>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    if state.intercept.drop_request(id) {
        Json(json!({ "id": id, "action": "drop" })).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn intercept_edit(
    Path(id): Path<u64>,
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<InterceptEditRequest>,
) -> impl IntoResponse {
    let body = if let Some(text) = payload.body {
        Some(text.into_bytes())
    } else if let Some(b64) = payload.body_base64 {
        match BASE64.decode(b64.as_bytes()) {
            Ok(bytes) => Some(bytes),
            Err(_) => {
                return (StatusCode::BAD_REQUEST, Json(json!({ "error": "invalid body_base64" })))
                    .into_response();
            }
        }
    } else {
        None
    };

    if state
        .intercept
        .edit(id, payload.method, payload.url, payload.headers, body)
    {
        Json(json!({ "id": id, "action": "edit" })).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

#[cfg(test)]
mod proxy_route_tests {
    use super::*;
    use crate::ip_filter::IpFilter;
    use crate::models::{AppSettings, ProxyConfig, UiConfig};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use interceptor_core::{
        capture::RequestCapture, cert_manager::CertManager, connection_pool::ConnectionPool,
        plugin::config::PluginSystemConfig, plugin::manager::PluginManager, rules::RuleEngine,
        InterceptQueue, Intruder, ProjectManager, ProxyController, Scanner, ScopeManager, WsCapture,
    };
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tower::util::ServiceExt;

    async fn test_state() -> Arc<AppState> {
        interceptor_core::connection_pool::install_crypto_provider();
        let capture = Arc::new(RequestCapture::new(100));
        let rules = Arc::new(RuleEngine::new());
        let scope = Arc::new(ScopeManager::new());
        let intercept = Arc::new(InterceptQueue::new());
        let scanner = Arc::new(Scanner::new());
        let plugin_manager = Arc::new(PluginManager::new(PluginSystemConfig::default()));
        let proxy = Arc::new(ProxyController::new(
            "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
            capture.clone(),
            rules.clone(),
            scope.clone(),
            None,
            Some(plugin_manager.clone()),
            Some(scanner.clone()),
            intercept.clone(),
        ));
        let cert_manager = Arc::new(CertManager::new().expect("cert manager"));
        let mut license_manager = interceptor_core::license::LicenseManager::new();
        let _ = license_manager.load_license();

        Arc::new(AppState {
            capture,
            cert_manager,
            pool: ConnectionPool::new(),
            rules,
            scope,
            intruder: Arc::new(Intruder::new()),
            scanner,
            ws_capture: Arc::new(WsCapture::new(100)),
            project_manager: Arc::new(ProjectManager::new(None)),
            api_token: None,
            max_body_bytes: 1024 * 1024,
            max_concurrency: 8,
            audit_logger: None,
            csrf_protection: None,
            ip_filter: Arc::new(IpFilter::new(crate::ip_filter::IpFilterConfig::default())),
            settings: Arc::new(RwLock::new(AppSettings {
                proxy: ProxyConfig {
                    port: 8080,
                    host: "127.0.0.1".into(),
                    intercept_https: false,
                    http2: false,
                },
                ui: UiConfig {
                    theme: "cyberpunk".into(),
                    animations: false,
                    notifications: false,
                },
            })),
            plugin_manager,
            license_manager: Arc::new(license_manager),
            proxy,
            intercept,
        })
    }

    #[tokio::test]
    async fn proxy_status_reports_stopped() {
        let state = test_state().await;
        let app = router().layer(Extension(state));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/proxy/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["running"], false);
        assert!(body.get("port").is_some());
        assert!(body.get("intercept_enabled").is_some());
    }

    #[tokio::test]
    async fn intercept_enable_and_list() {
        let state = test_state().await;
        let app = router().layer(Extension(state));
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/intercept")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"enabled":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["enabled"], true);
    }
}
