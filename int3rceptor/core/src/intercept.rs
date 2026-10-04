use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;

/// Snapshot of an in-scope request waiting for an operator decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeldRequest {
    pub id: u64,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub tls: bool,
    pub timestamp_ms: i128,
}

/// Operator decision for a held request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum InterceptDecision {
    Forward,
    Drop,
    Edit {
        method: Option<String>,
        url: Option<String>,
        headers: Option<Vec<(String, String)>>,
        body: Option<Vec<u8>>,
    },
}

struct PendingEntry {
    held: HeldRequest,
    decision_tx: oneshot::Sender<InterceptDecision>,
}

/// Queue that can hold in-scope proxied requests until forward / drop / edit.
#[derive(Clone, Default)]
pub struct InterceptQueue {
    inner: Arc<InterceptQueueInner>,
}

struct InterceptQueueInner {
    enabled: AtomicBool,
    next_id: AtomicU64,
    pending: DashMap<u64, PendingEntry>,
    drop_on_cancel: AtomicBool,
}

impl Default for InterceptQueueInner {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            next_id: AtomicU64::new(1),
            pending: DashMap::new(),
            drop_on_cancel: AtomicBool::new(false),
        }
    }
}

impl InterceptQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InterceptQueueInner::default()),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.inner.enabled.store(enabled, Ordering::SeqCst);
        if !enabled {
            // Auto-forward anything still held when intercept is turned off.
            self.flush_with(InterceptDecision::Forward);
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.enabled.load(Ordering::SeqCst)
    }

    pub fn len(&self) -> usize {
        self.inner.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.pending.is_empty()
    }

    pub fn list(&self) -> Vec<HeldRequest> {
        let mut items: Vec<_> = self
            .inner
            .pending
            .iter()
            .map(|entry| entry.held.clone())
            .collect();
        items.sort_by_key(|item| item.id);
        items
    }

    pub fn get(&self, id: u64) -> Option<HeldRequest> {
        self.inner
            .pending
            .get(&id)
            .map(|entry| entry.held.clone())
    }

    /// Apply a decision to a held request. Returns false if the id is unknown
    /// or the waiter already resolved.
    pub fn decide(&self, id: u64, decision: InterceptDecision) -> bool {
        let Some((_, entry)) = self.inner.pending.remove(&id) else {
            return false;
        };
        entry.decision_tx.send(decision).is_ok()
    }

    pub fn forward(&self, id: u64) -> bool {
        self.decide(id, InterceptDecision::Forward)
    }

    pub fn drop_request(&self, id: u64) -> bool {
        self.decide(id, InterceptDecision::Drop)
    }

    pub fn edit(
        &self,
        id: u64,
        method: Option<String>,
        url: Option<String>,
        headers: Option<Vec<(String, String)>>,
        body: Option<Vec<u8>>,
    ) -> bool {
        self.decide(
            id,
            InterceptDecision::Edit {
                method,
                url,
                headers,
                body,
            },
        )
    }

    /// Hold a request when intercept is enabled. When disabled, returns Forward
    /// immediately without queueing.
    pub async fn hold(&self, mut request: HeldRequest) -> InterceptDecision {
        if !self.is_enabled() {
            return InterceptDecision::Forward;
        }

        let id = self.inner.next_id.fetch_add(1, Ordering::SeqCst);
        request.id = id;
        let (tx, rx) = oneshot::channel();
        self.inner.pending.insert(
            id,
            PendingEntry {
                held: request,
                decision_tx: tx,
            },
        );

        match rx.await {
            Ok(decision) => decision,
            Err(_) => {
                if self.inner.drop_on_cancel.load(Ordering::SeqCst) {
                    InterceptDecision::Drop
                } else {
                    InterceptDecision::Forward
                }
            }
        }
    }

    /// Release all held requests with Forward (e.g. proxy stop).
    pub fn flush_forward(&self) {
        self.flush_with(InterceptDecision::Forward);
    }

    fn flush_with(&self, decision: InterceptDecision) {
        let ids: Vec<u64> = self.inner.pending.iter().map(|e| *e.key()).collect();
        for id in ids {
            let _ = self.decide(id, decision.clone());
        }
    }

    pub fn status(&self) -> InterceptStatus {
        InterceptStatus {
            enabled: self.is_enabled(),
            held_count: self.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptStatus {
    pub enabled: bool,
    pub held_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    fn sample_request(method: &str, url: &str) -> HeldRequest {
        HeldRequest {
            id: 0,
            method: method.to_string(),
            url: url.to_string(),
            headers: vec![("host".into(), "example.com".into())],
            body: b"hello".to_vec(),
            tls: false,
            timestamp_ms: 1_700_000_000_000,
        }
    }

    #[tokio::test]
    async fn disabled_queue_forwards_immediately() {
        let queue = InterceptQueue::new();
        assert!(!queue.is_enabled());
        let decision = queue.hold(sample_request("GET", "http://example.com/")).await;
        assert!(matches!(decision, InterceptDecision::Forward));
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn hold_and_forward_decision() {
        let queue = InterceptQueue::new();
        queue.set_enabled(true);

        let hold = tokio::spawn({
            let queue = queue.clone();
            async move { queue.hold(sample_request("POST", "http://example.com/api")).await }
        });

        timeout(Duration::from_secs(1), async {
            loop {
                if !queue.is_empty() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("request should be held");

        let held = queue.list();
        assert_eq!(held.len(), 1);
        assert_eq!(held[0].method, "POST");
        assert!(queue.forward(held[0].id));

        let decision = hold.await.expect("hold task");
        assert!(matches!(decision, InterceptDecision::Forward));
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn hold_and_drop_decision() {
        let queue = InterceptQueue::new();
        queue.set_enabled(true);

        let hold = tokio::spawn({
            let queue = queue.clone();
            async move { queue.hold(sample_request("GET", "http://example.com/drop")).await }
        });

        timeout(Duration::from_secs(1), async {
            loop {
                if !queue.is_empty() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let id = queue.list()[0].id;
        assert!(queue.drop_request(id));
        let decision = hold.await.unwrap();
        assert!(matches!(decision, InterceptDecision::Drop));
    }

    #[tokio::test]
    async fn hold_and_edit_decision() {
        let queue = InterceptQueue::new();
        queue.set_enabled(true);

        let hold = tokio::spawn({
            let queue = queue.clone();
            async move { queue.hold(sample_request("GET", "http://example.com/edit")).await }
        });

        timeout(Duration::from_secs(1), async {
            loop {
                if !queue.is_empty() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let id = queue.list()[0].id;
        assert!(queue.edit(
            id,
            Some("PUT".into()),
            Some("http://example.com/edited".into()),
            Some(vec![("x-test".into(), "1".into())]),
            Some(b"edited-body".to_vec()),
        ));

        match hold.await.unwrap() {
            InterceptDecision::Edit {
                method,
                url,
                headers,
                body,
            } => {
                assert_eq!(method.as_deref(), Some("PUT"));
                assert_eq!(url.as_deref(), Some("http://example.com/edited"));
                assert_eq!(headers, Some(vec![("x-test".into(), "1".into())]));
                assert_eq!(body.as_deref(), Some(b"edited-body".as_slice()));
            }
            other => panic!("expected edit, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn disabling_intercept_flushes_with_forward() {
        let queue = InterceptQueue::new();
        queue.set_enabled(true);

        let hold = tokio::spawn({
            let queue = queue.clone();
            async move { queue.hold(sample_request("GET", "http://example.com/flush")).await }
        });

        timeout(Duration::from_secs(1), async {
            loop {
                if !queue.is_empty() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        queue.set_enabled(false);
        let decision = hold.await.unwrap();
        assert!(matches!(decision, InterceptDecision::Forward));
        assert!(!queue.is_enabled());
        assert!(queue.is_empty());
    }

    #[test]
    fn decide_unknown_id_returns_false() {
        let queue = InterceptQueue::new();
        assert!(!queue.forward(999));
        assert!(!queue.drop_request(42));
    }
}
