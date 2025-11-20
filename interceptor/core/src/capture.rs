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
