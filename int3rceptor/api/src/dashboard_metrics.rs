//! `GET /api/dashboard/metrics`: the system metrics the dashboard polls.
//!
//! The core only keeps monotonic counters. Rates, including the error rate, are
//! derived from the difference between two consecutive samples, so they describe
//! recent traffic and not the whole life of the process.

use interceptor_core::metrics::MetricsSnapshot;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Samples closer than this reuse the previous window, so several clients polling
/// at once do not produce noisy or divide-by-near-zero rates, and the process
/// memory is not re-read on every request.
const MIN_SAMPLE_INTERVAL: Duration = Duration::from_millis(250);

/// Mirrors the `SystemMetrics` type used by the frontend.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DashboardMetrics {
    /// Unix time in milliseconds.
    pub timestamp: i64,
    pub requests_per_sec: f64,
    pub avg_response_time_ms: f64,
    /// Resident memory of this process in MiB. 0 where it cannot be read.
    pub memory_usage_mb: f64,
    pub active_connections: u64,
    pub bytes_in_sec: f64,
    pub bytes_out_sec: f64,
    /// Share of requests that failed among those completed since the previous
    /// sample (0 to 100). It is 0 while no request completes.
    pub error_rate_percent: f64,
    pub uptime_seconds: u64,
}

/// Values computed once per sample window and reused until the next window.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Window {
    requests_per_sec: f64,
    bytes_in_sec: f64,
    bytes_out_sec: f64,
    error_rate_percent: f64,
    memory_usage_mb: f64,
}

#[derive(Debug, Clone, Copy)]
struct Sample {
    at: Instant,
    requests_total: u64,
    requests_completed: u64,
    requests_error: u64,
    bytes_received: u64,
    bytes_sent: u64,
    window: Window,
}

impl Sample {
    fn new(snapshot: &MetricsSnapshot, at: Instant, window: Window) -> Self {
        Self {
            at,
            requests_total: snapshot.requests_total,
            requests_completed: snapshot.requests_success + snapshot.requests_error,
            requests_error: snapshot.requests_error,
            bytes_received: snapshot.bytes_received,
            bytes_sent: snapshot.bytes_sent,
            window,
        }
    }
}

/// Keeps the previous sample so rates can be computed between polls.
#[derive(Default)]
pub struct RateSampler {
    last: Mutex<Option<Sample>>,
}

impl RateSampler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process-wide sampler. The core metrics are a global singleton too, so rates
    /// are per process rather than per `AppState`.
    pub fn global() -> &'static RateSampler {
        static SAMPLER: OnceLock<RateSampler> = OnceLock::new();
        SAMPLER.get_or_init(RateSampler::new)
    }

    pub fn sample(&self, snapshot: &MetricsSnapshot, now: Instant) -> DashboardMetrics {
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        let window = match *last {
            Some(prev) if now.duration_since(prev.at) < MIN_SAMPLE_INTERVAL => prev.window,
            Some(prev) => {
                let window = compute_window(&prev, snapshot, now, process_memory_mb);
                *last = Some(Sample::new(snapshot, now, window));
                window
            }
            None => {
                let window = Window {
                    memory_usage_mb: process_memory_mb(),
                    ..Window::default()
                };
                *last = Some(Sample::new(snapshot, now, window));
                window
            }
        };
        build(snapshot, window)
    }
}

fn compute_window(
    prev: &Sample,
    snapshot: &MetricsSnapshot,
    now: Instant,
    read_memory_mb: impl FnOnce() -> f64,
) -> Window {
    let secs = now.duration_since(prev.at).as_secs_f64().max(f64::EPSILON);
    // Counters reset to zero when metrics are reset; treat that as no traffic.
    let delta = |current: u64, before: u64| current.saturating_sub(before);
    let per_sec = |current: u64, before: u64| delta(current, before) as f64 / secs;

    let completed = delta(
        snapshot.requests_success + snapshot.requests_error,
        prev.requests_completed,
    );
    let errors = delta(snapshot.requests_error, prev.requests_error);
    let error_rate_percent = if completed == 0 {
        0.0
    } else {
        (errors as f64 / completed as f64 * 100.0).min(100.0)
    };

    Window {
        requests_per_sec: per_sec(snapshot.requests_total, prev.requests_total),
        bytes_in_sec: per_sec(snapshot.bytes_received, prev.bytes_received),
        bytes_out_sec: per_sec(snapshot.bytes_sent, prev.bytes_sent),
        error_rate_percent,
        memory_usage_mb: read_memory_mb(),
    }
}

fn build(snapshot: &MetricsSnapshot, window: Window) -> DashboardMetrics {
    DashboardMetrics {
        timestamp: time::OffsetDateTime::now_utc().unix_timestamp() * 1000,
        requests_per_sec: window.requests_per_sec,
        avg_response_time_ms: snapshot.avg_request_duration_us as f64 / 1000.0,
        memory_usage_mb: window.memory_usage_mb,
        active_connections: snapshot.connections_active,
        bytes_in_sec: window.bytes_in_sec,
        bytes_out_sec: window.bytes_out_sec,
        error_rate_percent: window.error_rate_percent,
        uptime_seconds: snapshot.uptime_secs,
    }
}

/// Resident set size of the current process in MiB (Linux only).
///
/// `/proc` is served from memory, so this is a short synchronous read. It runs at
/// most once per sample window, not once per request.
fn process_memory_mb() -> f64 {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return 0.0;
    };
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|kb| kb.parse::<f64>().ok())
        .map_or(0.0, |kb| kb / 1024.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn snapshot(requests: u64, ok: u64, errors: u64, received: u64, sent: u64) -> MetricsSnapshot {
        let mut s = interceptor_core::metrics::Metrics::new().snapshot();
        s.requests_total = requests;
        s.requests_success = ok;
        s.requests_error = errors;
        s.bytes_received = received;
        s.bytes_sent = sent;
        s
    }

    #[test]
    fn first_sample_has_zero_rates() {
        let sampler = RateSampler::new();
        let m = sampler.sample(&snapshot(100, 90, 10, 1000, 2000), Instant::now());
        assert_eq!(m.requests_per_sec, 0.0);
        assert_eq!(m.bytes_in_sec, 0.0);
        assert_eq!(m.bytes_out_sec, 0.0);
        assert_eq!(m.error_rate_percent, 0.0);
    }

    #[test]
    fn rates_come_from_the_delta_between_samples() {
        let sampler = RateSampler::new();
        let t0 = Instant::now();
        sampler.sample(&snapshot(100, 100, 0, 1000, 2000), t0);
        let m = sampler.sample(
            &snapshot(120, 120, 0, 3000, 2500),
            t0 + Duration::from_secs(2),
        );
        assert_eq!(m.requests_per_sec, 10.0);
        assert_eq!(m.bytes_in_sec, 1000.0);
        assert_eq!(m.bytes_out_sec, 250.0);
    }

    #[test]
    fn counter_reset_is_not_negative_traffic() {
        let sampler = RateSampler::new();
        let t0 = Instant::now();
        sampler.sample(&snapshot(500, 400, 100, 9000, 9000), t0);
        let m = sampler.sample(&snapshot(3, 3, 0, 10, 10), t0 + Duration::from_secs(1));
        assert_eq!(m.requests_per_sec, 0.0);
        assert_eq!(m.bytes_in_sec, 0.0);
        assert_eq!(m.error_rate_percent, 0.0);
    }

    #[test]
    fn fast_polls_reuse_the_previous_window() {
        let sampler = RateSampler::new();
        let t0 = Instant::now();
        sampler.sample(&snapshot(0, 0, 0, 0, 0), t0);
        let first = sampler.sample(&snapshot(10, 10, 0, 0, 0), t0 + Duration::from_secs(1));
        let second = sampler.sample(&snapshot(11, 11, 0, 0, 0), t0 + Duration::from_millis(1050));
        assert_eq!(first.requests_per_sec, 10.0);
        assert_eq!(second.requests_per_sec, 10.0);
        assert_eq!(second.memory_usage_mb, first.memory_usage_mb);
    }

    #[test]
    fn error_rate_covers_only_the_latest_window() {
        let sampler = RateSampler::new();
        let t0 = Instant::now();
        sampler.sample(&snapshot(0, 0, 0, 0, 0), t0);

        // 50 of 200 completed requests failed in the first window.
        let busy = sampler.sample(&snapshot(200, 150, 50, 0, 0), t0 + Duration::from_secs(1));
        assert_eq!(busy.error_rate_percent, 25.0);

        // The next window has 100 more requests and no new errors: the old
        // failures must no longer show up.
        let clean = sampler.sample(&snapshot(300, 250, 50, 0, 0), t0 + Duration::from_secs(2));
        assert_eq!(clean.error_rate_percent, 0.0);
    }

    #[test]
    fn error_rate_is_zero_while_nothing_completes() {
        let sampler = RateSampler::new();
        let t0 = Instant::now();
        sampler.sample(&snapshot(10, 10, 0, 0, 0), t0);
        // Requests started but none finished yet (in flight).
        let m = sampler.sample(&snapshot(20, 10, 0, 0, 0), t0 + Duration::from_secs(1));
        assert_eq!(m.error_rate_percent, 0.0);
    }

    #[test]
    fn memory_is_read_once_per_window() {
        let reads = Cell::new(0);
        let t0 = Instant::now();
        let prev = Sample::new(&snapshot(0, 0, 0, 0, 0), t0, Window::default());
        let window = compute_window(
            &prev,
            &snapshot(1, 1, 0, 0, 0),
            t0 + Duration::from_secs(1),
            || {
                reads.set(reads.get() + 1);
                42.0
            },
        );
        assert_eq!(reads.get(), 1);
        assert_eq!(window.memory_usage_mb, 42.0);
    }
}
