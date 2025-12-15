use crate::{
    models::{
        AppSettings, HeaderPatch, ManualRequest, ManualResponse, PluginInfo, PluginToggle,
        RepeatRequest,
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
use interceptor_core::plugin::config::{PluginConfig, PluginPermissions};
use interceptor_core::rules::Rule;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

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
        // Metrics and monitoring
        .route("/api/metrics", get(get_metrics))
        .route("/api/metrics/reset", post(reset_metrics))
        .route("/api/health", get(health_check))
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

    let body_bytes = payload
        .modified_body
        .map(|b| b.into_bytes())
        .unwrap_or(entry.request.body.clone());

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
    state.capture.clear();
    StatusCode::NO_CONTENT
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
        Ok(requests) => Json(json!({ "requests": requests })).into_response(),
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

async fn send_manual_request(Json(payload): Json<ManualRequest>) -> impl IntoResponse {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_default();

    let method = match payload.method.parse::<reqwest::Method>() {
        Ok(m) => m,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let start = Instant::now();

    let mut req_builder = client.request(method, &payload.url);

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
