//! Metrics collection for INT3RCEPTOR
//!
//! Provides thread-safe counters and gauges for monitoring proxy performance.

use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Global metrics instance
static METRICS: once_cell::sync::Lazy<Metrics> = once_cell::sync::Lazy::new(Metrics::new);

/// Get the global metrics instance
pub fn metrics() -> &'static Metrics {
    &METRICS
}

/// Core metrics collection
#[derive(Debug)]
pub struct Metrics {
    // Request counters
    requests_total: AtomicU64,
    requests_success: AtomicU64,
    requests_error: AtomicU64,
    requests_intercepted: AtomicU64,

    // Response status counters
    responses_2xx: AtomicU64,
    responses_3xx: AtomicU64,
    responses_4xx: AtomicU64,
    responses_5xx: AtomicU64,

    // Connection counters
    connections_active: AtomicU64,
    connections_total: AtomicU64,
    websocket_connections: AtomicU64,

    // Bytes transferred
    bytes_received: AtomicU64,
    bytes_sent: AtomicU64,

    // Timing (stored as microseconds)
    request_duration_sum_us: AtomicU64,
    request_duration_count: AtomicU64,
    request_duration_max_us: AtomicU64,

    // TLS stats
    tls_handshakes: AtomicU64,
    tls_errors: AtomicU64,

    // Rule engine stats
    rules_applied: AtomicU64,
    rules_matched: AtomicU64,

    // Start time for uptime calculation
    start_time: Instant,

    // Histogram buckets for latency distribution (in ms)
    latency_buckets: RwLock<LatencyHistogram>,

    // Per-host request counts
    host_requests: RwLock<HashMap<String, u64>>,
}

impl Metrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_error: AtomicU64::new(0),
            requests_intercepted: AtomicU64::new(0),

            responses_2xx: AtomicU64::new(0),
            responses_3xx: AtomicU64::new(0),
            responses_4xx: AtomicU64::new(0),
            responses_5xx: AtomicU64::new(0),

            connections_active: AtomicU64::new(0),
            connections_total: AtomicU64::new(0),
            websocket_connections: AtomicU64::new(0),

            bytes_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),

            request_duration_sum_us: AtomicU64::new(0),
            request_duration_count: AtomicU64::new(0),
            request_duration_max_us: AtomicU64::new(0),

            tls_handshakes: AtomicU64::new(0),
            tls_errors: AtomicU64::new(0),

            rules_applied: AtomicU64::new(0),
            rules_matched: AtomicU64::new(0),

            start_time: Instant::now(),

            latency_buckets: RwLock::new(LatencyHistogram::new()),
            host_requests: RwLock::new(HashMap::new()),
        }
    }

    // ==================== Request Tracking ====================

    /// Record a new request
    pub fn record_request(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful request
    pub fn record_request_success(&self) {
        self.requests_success.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed request
    pub fn record_request_error(&self) {
        self.requests_error.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an intercepted (modified) request
    pub fn record_request_intercepted(&self) {
        self.requests_intercepted.fetch_add(1, Ordering::Relaxed);
    }

    /// Record request for a specific host
    pub fn record_host_request(&self, host: &str) {
        let mut hosts = self.host_requests.write();
        *hosts.entry(host.to_string()).or_insert(0) += 1;
    }

    // ==================== Response Tracking ====================

    /// Record a response by status code
    pub fn record_response(&self, status_code: u16) {
        match status_code {
            200..=299 => self.responses_2xx.fetch_add(1, Ordering::Relaxed),
            300..=399 => self.responses_3xx.fetch_add(1, Ordering::Relaxed),
            400..=499 => self.responses_4xx.fetch_add(1, Ordering::Relaxed),
            500..=599 => self.responses_5xx.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }

    // ==================== Connection Tracking ====================

    /// Increment active connections
    pub fn connection_opened(&self) {
        self.connections_active.fetch_add(1, Ordering::Relaxed);
        self.connections_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn connection_closed(&self) {
        self.connections_active.fetch_sub(1, Ordering::Relaxed);
    }

    /// Increment WebSocket connections
    pub fn websocket_opened(&self) {
        self.websocket_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement WebSocket connections
    pub fn websocket_closed(&self) {
        self.websocket_connections.fetch_sub(1, Ordering::Relaxed);
    }

    // ==================== Bytes Tracking ====================

    /// Record bytes received
    pub fn record_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record bytes sent
    pub fn record_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    // ==================== Timing Tracking ====================

    /// Record request duration
    pub fn record_request_duration(&self, duration: Duration) {
        let micros = duration.as_micros() as u64;
        self.request_duration_sum_us.fetch_add(micros, Ordering::Relaxed);
        self.request_duration_count.fetch_add(1, Ordering::Relaxed);

        // Update max (compare-and-swap loop)
        let mut current_max = self.request_duration_max_us.load(Ordering::Relaxed);
        while micros > current_max {
            match self.request_duration_max_us.compare_exchange_weak(
                current_max,
                micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }

        // Record in histogram
        self.latency_buckets.write().record(duration);
    }

    /// Start timing a request (returns guard that records on drop)
    pub fn time_request(&self) -> RequestTimer<'_> {
        RequestTimer {
            metrics: self,
            start: Instant::now(),
        }
    }

    // ==================== TLS Tracking ====================

    /// Record a TLS handshake
    pub fn record_tls_handshake(&self) {
        self.tls_handshakes.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a TLS error
    pub fn record_tls_error(&self) {
        self.tls_errors.fetch_add(1, Ordering::Relaxed);
    }

    // ==================== Rule Engine Tracking ====================

    /// Record rules being applied
    pub fn record_rules_applied(&self, count: u64) {
        self.rules_applied.fetch_add(count, Ordering::Relaxed);
    }

    /// Record rules that matched
    pub fn record_rules_matched(&self, count: u64) {
        self.rules_matched.fetch_add(count, Ordering::Relaxed);
    }

    // ==================== Snapshot ====================

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let duration_count = self.request_duration_count.load(Ordering::Relaxed);
        let duration_sum = self.request_duration_sum_us.load(Ordering::Relaxed);
        let avg_duration_us = if duration_count > 0 {
            duration_sum / duration_count
        } else {
            0
        };

        MetricsSnapshot {
            uptime_secs: self.start_time.elapsed().as_secs(),

            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success: self.requests_success.load(Ordering::Relaxed),
            requests_error: self.requests_error.load(Ordering::Relaxed),
            requests_intercepted: self.requests_intercepted.load(Ordering::Relaxed),

            responses_2xx: self.responses_2xx.load(Ordering::Relaxed),
            responses_3xx: self.responses_3xx.load(Ordering::Relaxed),
            responses_4xx: self.responses_4xx.load(Ordering::Relaxed),
            responses_5xx: self.responses_5xx.load(Ordering::Relaxed),

            connections_active: self.connections_active.load(Ordering::Relaxed),
            connections_total: self.connections_total.load(Ordering::Relaxed),
            websocket_connections: self.websocket_connections.load(Ordering::Relaxed),

            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),

            avg_request_duration_us: avg_duration_us,
            max_request_duration_us: self.request_duration_max_us.load(Ordering::Relaxed),

            tls_handshakes: self.tls_handshakes.load(Ordering::Relaxed),
            tls_errors: self.tls_errors.load(Ordering::Relaxed),

            rules_applied: self.rules_applied.load(Ordering::Relaxed),
            rules_matched: self.rules_matched.load(Ordering::Relaxed),

            latency_histogram: self.latency_buckets.read().snapshot(),
            top_hosts: self.top_hosts(10),
        }
    }

    /// Get top N hosts by request count
    pub fn top_hosts(&self, n: usize) -> Vec<(String, u64)> {
        let hosts = self.host_requests.read();
        let mut vec: Vec<_> = hosts.iter().map(|(k, v)| (k.clone(), *v)).collect();
        vec.sort_by(|a, b| b.1.cmp(&a.1));
        vec.truncate(n);
        vec
    }

    /// Reset all counters
    pub fn reset(&self) {
        self.requests_total.store(0, Ordering::Relaxed);
        self.requests_success.store(0, Ordering::Relaxed);
        self.requests_error.store(0, Ordering::Relaxed);
        self.requests_intercepted.store(0, Ordering::Relaxed);

        self.responses_2xx.store(0, Ordering::Relaxed);
        self.responses_3xx.store(0, Ordering::Relaxed);
        self.responses_4xx.store(0, Ordering::Relaxed);
        self.responses_5xx.store(0, Ordering::Relaxed);

        self.bytes_received.store(0, Ordering::Relaxed);
        self.bytes_sent.store(0, Ordering::Relaxed);

        self.request_duration_sum_us.store(0, Ordering::Relaxed);
        self.request_duration_count.store(0, Ordering::Relaxed);
        self.request_duration_max_us.store(0, Ordering::Relaxed);

        self.tls_handshakes.store(0, Ordering::Relaxed);
        self.tls_errors.store(0, Ordering::Relaxed);

        self.rules_applied.store(0, Ordering::Relaxed);
        self.rules_matched.store(0, Ordering::Relaxed);

        self.latency_buckets.write().reset();
        self.host_requests.write().clear();
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII timer for request duration
pub struct RequestTimer<'a> {
    metrics: &'a Metrics,
    start: Instant,
}

impl Drop for RequestTimer<'_> {
    fn drop(&mut self) {
        self.metrics.record_request_duration(self.start.elapsed());
    }
}

/// Latency histogram with predefined buckets
#[derive(Debug)]
pub struct LatencyHistogram {
    // Buckets: <1ms, <5ms, <10ms, <25ms, <50ms, <100ms, <250ms, <500ms, <1s, <5s, >5s
    buckets: [u64; 11],
}

impl LatencyHistogram {
    const BUCKET_BOUNDS_MS: [u64; 10] = [1, 5, 10, 25, 50, 100, 250, 500, 1000, 5000];

    pub fn new() -> Self {
        Self { buckets: [0; 11] }
    }

    pub fn record(&mut self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        let bucket = Self::BUCKET_BOUNDS_MS
            .iter()
            .position(|&bound| ms < bound)
            .unwrap_or(10);
        self.buckets[bucket] += 1;
    }

    pub fn snapshot(&self) -> LatencyHistogramSnapshot {
        LatencyHistogramSnapshot {
            lt_1ms: self.buckets[0],
            lt_5ms: self.buckets[1],
            lt_10ms: self.buckets[2],
            lt_25ms: self.buckets[3],
            lt_50ms: self.buckets[4],
            lt_100ms: self.buckets[5],
            lt_250ms: self.buckets[6],
            lt_500ms: self.buckets[7],
            lt_1s: self.buckets[8],
            lt_5s: self.buckets[9],
            gt_5s: self.buckets[10],
        }
    }

    pub fn reset(&mut self) {
        self.buckets = [0; 11];
    }
}

impl Default for LatencyHistogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable snapshot of latency histogram
#[derive(Debug, Clone, Serialize)]
pub struct LatencyHistogramSnapshot {
    pub lt_1ms: u64,
    pub lt_5ms: u64,
    pub lt_10ms: u64,
    pub lt_25ms: u64,
    pub lt_50ms: u64,
    pub lt_100ms: u64,
    pub lt_250ms: u64,
    pub lt_500ms: u64,
    pub lt_1s: u64,
    pub lt_5s: u64,
    pub gt_5s: u64,
}

/// Serializable snapshot of all metrics
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub uptime_secs: u64,

    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub requests_intercepted: u64,

    pub responses_2xx: u64,
    pub responses_3xx: u64,
    pub responses_4xx: u64,
    pub responses_5xx: u64,

    pub connections_active: u64,
    pub connections_total: u64,
    pub websocket_connections: u64,

    pub bytes_received: u64,
    pub bytes_sent: u64,

    pub avg_request_duration_us: u64,
    pub max_request_duration_us: u64,

    pub tls_handshakes: u64,
    pub tls_errors: u64,

    pub rules_applied: u64,
    pub rules_matched: u64,

    pub latency_histogram: LatencyHistogramSnapshot,
    pub top_hosts: Vec<(String, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_counters() {
        let m = Metrics::new();

        m.record_request();
        m.record_request();
        m.record_request_success();
        m.record_request_error();

        let snap = m.snapshot();
        assert_eq!(snap.requests_total, 2);
        assert_eq!(snap.requests_success, 1);
        assert_eq!(snap.requests_error, 1);
    }

    #[test]
    fn test_response_counters() {
        let m = Metrics::new();

        m.record_response(200);
        m.record_response(201);
        m.record_response(301);
        m.record_response(404);
        m.record_response(500);

        let snap = m.snapshot();
        assert_eq!(snap.responses_2xx, 2);
        assert_eq!(snap.responses_3xx, 1);
        assert_eq!(snap.responses_4xx, 1);
        assert_eq!(snap.responses_5xx, 1);
    }

    #[test]
    fn test_connection_tracking() {
        let m = Metrics::new();

        m.connection_opened();
        m.connection_opened();
        m.connection_closed();

        let snap = m.snapshot();
        assert_eq!(snap.connections_active, 1);
        assert_eq!(snap.connections_total, 2);
    }

    #[test]
    fn test_bytes_tracking() {
        let m = Metrics::new();

        m.record_bytes_received(1000);
        m.record_bytes_sent(500);
        m.record_bytes_received(500);

        let snap = m.snapshot();
        assert_eq!(snap.bytes_received, 1500);
        assert_eq!(snap.bytes_sent, 500);
    }

    #[test]
    fn test_request_duration() {
        let m = Metrics::new();

        m.record_request_duration(Duration::from_millis(10));
        m.record_request_duration(Duration::from_millis(20));
        m.record_request_duration(Duration::from_millis(30));

        let snap = m.snapshot();
        assert_eq!(snap.avg_request_duration_us, 20_000); // 20ms average
        assert_eq!(snap.max_request_duration_us, 30_000); // 30ms max
    }

    #[test]
    fn test_request_timer() {
        let m = Metrics::new();

        {
            let _timer = m.time_request();
            std::thread::sleep(Duration::from_millis(5));
        }

        let snap = m.snapshot();
        assert!(snap.avg_request_duration_us >= 5_000); // At least 5ms
    }

    #[test]
    fn test_latency_histogram() {
        let mut h = LatencyHistogram::new();

        h.record(Duration::from_micros(500)); // <1ms bucket
        h.record(Duration::from_millis(3));   // <5ms bucket
        h.record(Duration::from_millis(50));  // <100ms bucket
        h.record(Duration::from_secs(10));    // >5s bucket

        let snap = h.snapshot();
        assert_eq!(snap.lt_1ms, 1);
        assert_eq!(snap.lt_5ms, 1);
        assert_eq!(snap.lt_100ms, 1);
        assert_eq!(snap.gt_5s, 1);
    }

    #[test]
    fn test_host_tracking() {
        let m = Metrics::new();

        m.record_host_request("example.com");
        m.record_host_request("example.com");
        m.record_host_request("test.com");
        m.record_host_request("example.com");

        let top = m.top_hosts(10);
        assert_eq!(top[0], ("example.com".to_string(), 3));
        assert_eq!(top[1], ("test.com".to_string(), 1));
    }

    #[test]
    fn test_reset() {
        let m = Metrics::new();

        m.record_request();
        m.record_response(200);
        m.record_bytes_received(100);

        m.reset();

        let snap = m.snapshot();
        assert_eq!(snap.requests_total, 0);
        assert_eq!(snap.responses_2xx, 0);
        assert_eq!(snap.bytes_received, 0);
    }

    #[test]
    fn test_global_metrics() {
        let m = metrics();
        m.record_request();
        assert!(m.snapshot().requests_total >= 1);
    }
}
