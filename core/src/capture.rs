use crate::storage::CaptureStorage;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::broadcast;

const DEFAULT_CAPACITY: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub id: u64,
    pub timestamp_ms: i128,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub tls: bool,
}

impl CapturedRequest {
    pub fn new(method: impl Into<String>, url: impl Into<String>, tls: bool) -> Self {
        Self {
            id: 0,
            timestamp_ms: OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000,
            method: method.into(),
            url: url.into(),
            headers: Vec::new(),
            body: Vec::new(),
            tls,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedResponse {
    pub request_id: u64,
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureEntry {
    pub request: CapturedRequest,
    pub response: Option<CapturedResponse>,
}

#[derive(Debug)]
pub struct RequestCapture {
    capacity: usize,
    entries: RwLock<VecDeque<CaptureEntry>>,
    counter: AtomicU64,
    notifier: broadcast::Sender<CaptureEntry>,
    storage: Option<Arc<CaptureStorage>>,
}

impl RequestCapture {
    pub fn new(capacity: usize) -> Self {
        Self::with_storage(capacity, None)
    }

    pub fn with_storage(capacity: usize, storage: Option<Arc<CaptureStorage>>) -> Self {
        let (tx, _) = broadcast::channel(1_024);
        Self {
            capacity: if capacity == 0 {
                DEFAULT_CAPACITY
            } else {
                capacity
            },
            entries: RwLock::new(VecDeque::new()),
            counter: AtomicU64::new(1),
            notifier: tx,
            storage,
        }
    }

    pub fn push(
        &self,
        mut request: CapturedRequest,
        mut response: Option<CapturedResponse>,
    ) -> u64 {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        request.id = id;
        if let Some(resp) = response.as_mut() {
            resp.request_id = id;
        }
        let entry = CaptureEntry { request, response };
        let notify = entry.clone();
        let mut guard = self.entries.write();
        guard.push_front(entry);
        while guard.len() > self.capacity {
            guard.pop_back();
        }
        drop(guard);
        let _ = self.notifier.send(notify.clone());
        if let Some(storage) = &self.storage {
            if let Err(err) = storage.insert(&notify) {
                tracing::warn!("persist_capture_error" = %err);
            }
        }
        id
    }

    pub fn get(&self, id: u64) -> Option<CaptureEntry> {
        let guard = self.entries.read();
        guard.iter().find(|item| item.request.id == id).cloned()
    }

    pub fn get_all(&self) -> Vec<CaptureEntry> {
        self.entries.read().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.entries.write().clear();
        if let Some(storage) = &self.storage {
            if let Err(err) = storage.clear() {
                tracing::warn!("clear_storage_error" = %err);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CaptureEntry> {
        self.notifier.subscribe()
    }

    pub fn query(&self, filter: &CaptureQuery) -> Vec<CaptureEntry> {
        if let Some(storage) = &self.storage {
            return storage.query(filter).unwrap_or_default();
        }
        let guard = self.entries.read();
        guard
            .iter()
            .filter(|entry| filter.matches(entry))
            .take(filter.limit.unwrap_or(guard.len()))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct CaptureQuery {
    pub method: Option<String>,
    pub host: Option<String>,
    pub status: Option<u16>,
    pub tls: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

impl CaptureQuery {
    fn matches(&self, entry: &CaptureEntry) -> bool {
        if let Some(method) = &self.method {
            if &entry.request.method != method {
                return false;
            }
        }
        if let Some(host) = &self.host {
            if !entry.request.url.contains(host) {
                return false;
            }
        }
        if let Some(status) = self.status {
            if entry
                .response
                .as_ref()
                .map(|r| r.status_code)
                .unwrap_or_default()
                != status
            {
                return false;
            }
        }
        if let Some(tls) = self.tls {
            if entry.request.tls != tls {
                return false;
            }
        }
        if let Some(search) = &self.search {
            if !entry.request.url.contains(search) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request(method: &str, url: &str, tls: bool) -> CapturedRequest {
        CapturedRequest::new(method, url, tls)
    }

    fn create_test_response(status: u16, duration: u128) -> CapturedResponse {
        CapturedResponse {
            request_id: 0,
            status_code: status,
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: b"{}".to_vec(),
            duration_ms: duration,
        }
    }

    #[test]
    fn test_captured_request_new() {
        let req = CapturedRequest::new("GET", "https://example.com/api", true);
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://example.com/api");
        assert!(req.tls);
        assert_eq!(req.id, 0);
        assert!(req.timestamp_ms > 0);
    }

    #[test]
    fn test_request_capture_push_and_get() {
        let capture = RequestCapture::new(100);

        let req = create_test_request("POST", "https://api.test.com/users", true);
        let resp = Some(create_test_response(201, 50));

        let id = capture.push(req, resp);
        assert!(id > 0);

        let entry = capture.get(id).unwrap();
        assert_eq!(entry.request.method, "POST");
        assert_eq!(entry.response.unwrap().status_code, 201);
    }

    #[test]
    fn test_request_capture_capacity() {
        let capture = RequestCapture::new(3);

        for i in 0..5 {
            let req = create_test_request("GET", &format!("/page/{}", i), false);
            capture.push(req, None);
        }

        // Should only keep 3 most recent
        assert_eq!(capture.len(), 3);

        // Oldest entries should be evicted (0 and 1)
        let all = capture.get_all();
        assert!(!all.iter().any(|e| e.request.url == "/page/0"));
        assert!(!all.iter().any(|e| e.request.url == "/page/1"));
    }

    #[test]
    fn test_request_capture_clear() {
        let capture = RequestCapture::new(100);

        capture.push(create_test_request("GET", "/test", false), None);
        capture.push(create_test_request("POST", "/submit", false), None);

        assert_eq!(capture.len(), 2);
        assert!(!capture.is_empty());

        capture.clear();

        assert_eq!(capture.len(), 0);
        assert!(capture.is_empty());
    }

    #[test]
    fn test_capture_query_method_filter() {
        let capture = RequestCapture::new(100);

        capture.push(create_test_request("GET", "/api/users", false), None);
        capture.push(create_test_request("POST", "/api/users", false), None);
        capture.push(create_test_request("GET", "/api/posts", false), None);

        let query = CaptureQuery {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        let results = capture.query(&query);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.request.method == "GET"));
    }

    #[test]
    fn test_capture_query_host_filter() {
        let capture = RequestCapture::new(100);

        capture.push(create_test_request("GET", "https://api.example.com/users", true), None);
        capture.push(create_test_request("GET", "https://other.com/data", true), None);
        capture.push(create_test_request("GET", "https://api.example.com/posts", true), None);

        let query = CaptureQuery {
            host: Some("example.com".to_string()),
            ..Default::default()
        };

        let results = capture.query(&query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_capture_query_status_filter() {
        let capture = RequestCapture::new(100);

        capture.push(
            create_test_request("GET", "/success", false),
            Some(create_test_response(200, 10)),
        );
        capture.push(
            create_test_request("GET", "/not-found", false),
            Some(create_test_response(404, 5)),
        );
        capture.push(
            create_test_request("GET", "/error", false),
            Some(create_test_response(500, 100)),
        );

        let query = CaptureQuery {
            status: Some(200),
            ..Default::default()
        };

        let results = capture.query(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].response.as_ref().unwrap().status_code, 200);
    }

    #[test]
    fn test_capture_query_tls_filter() {
        let capture = RequestCapture::new(100);

        capture.push(create_test_request("GET", "https://secure.com", true), None);
        capture.push(create_test_request("GET", "http://insecure.com", false), None);
        capture.push(create_test_request("GET", "https://another.com", true), None);

        let query = CaptureQuery {
            tls: Some(true),
            ..Default::default()
        };

        let results = capture.query(&query);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.request.tls));
    }

    #[test]
    fn test_capture_query_search_filter() {
        let capture = RequestCapture::new(100);

        capture.push(create_test_request("GET", "/api/v1/users", false), None);
        capture.push(create_test_request("GET", "/api/v2/posts", false), None);
        capture.push(create_test_request("GET", "/web/home", false), None);

        let query = CaptureQuery {
            search: Some("/api/".to_string()),
            ..Default::default()
        };

        let results = capture.query(&query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_capture_query_limit() {
        let capture = RequestCapture::new(100);

        for i in 0..10 {
            capture.push(create_test_request("GET", &format!("/page/{}", i), false), None);
        }

        let query = CaptureQuery {
            limit: Some(3),
            ..Default::default()
        };

        let results = capture.query(&query);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_capture_query_combined_filters() {
        let capture = RequestCapture::new(100);

        capture.push(
            create_test_request("GET", "https://api.test.com/users", true),
            Some(create_test_response(200, 10)),
        );
        capture.push(
            create_test_request("POST", "https://api.test.com/users", true),
            Some(create_test_response(201, 20)),
        );
        capture.push(
            create_test_request("GET", "https://other.com/data", true),
            Some(create_test_response(200, 15)),
        );

        let query = CaptureQuery {
            method: Some("GET".to_string()),
            host: Some("test.com".to_string()),
            status: Some(200),
            ..Default::default()
        };

        let results = capture.query(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].request.url, "https://api.test.com/users");
    }

    #[test]
    fn test_default_capacity() {
        let capture = RequestCapture::new(0);
        // 0 should use DEFAULT_CAPACITY (10_000)
        for _ in 0..100 {
            capture.push(create_test_request("GET", "/test", false), None);
        }
        assert_eq!(capture.len(), 100);
    }

    #[test]
    fn test_broadcast_subscription() {
        let capture = RequestCapture::new(100);
        let mut receiver = capture.subscribe();

        capture.push(create_test_request("GET", "/notify", false), None);

        // Should receive the entry
        let result = receiver.try_recv();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().request.url, "/notify");
    }
}
