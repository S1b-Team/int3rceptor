use crate::{
    models::{HeaderPatch, RepeatRequest},
    state::AppState,
};
use axum::extract::{Extension, Path, Query};
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
use interceptor_core::connection_pool::ProxyBody;
use interceptor_core::rules::Rule;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub fn router() -> Router {
    Router::new()
        .route("/api/requests", get(list_requests).delete(clear_requests))
        .route("/api/requests/:id", get(get_request))
        .route("/api/requests/:id/repeat", post(repeat_request))
        .route("/api/requests/export", get(export_requests))
        .route("/api/ca-cert", get(download_ca_cert))
        .route(
            "/api/rules",
            get(list_rules).post(add_rule).delete(clear_rules),
        )
        .route("/api/scope", get(get_scope).put(set_scope))
        .route("/api/intruder/generate", post(intruder_generate))
        .route("/api/intruder/results", get(intruder_results).delete(intruder_clear))
        .route("/api/websocket/connections", get(ws_connections))
        .route("/api/websocket/frames/:connection_id", get(ws_frames))
        .route("/api/websocket/clear", delete(ws_clear))
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

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ExportFormat {
    Json,
    Csv,
    Har,
}

impl Default for ExportFormat {
    fn default() -> Self {
        ExportFormat::Json
    }
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
async fn get_scope(Extension(state): Extension<Arc<AppState>>) -> Json<interceptor_core::scope::ScopeConfig> {
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
