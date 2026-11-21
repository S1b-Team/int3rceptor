//! `GET /api/dashboard/metrics`: the system metrics the dashboard polls.
//!
//! The core only keeps monotonic counters. Per-second rates are derived here from
//! the difference between two consecutive samples.

use interceptor_core::metrics::MetricsSnapshot;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Samples closer than this reuse the previous rates, so several clients polling
/// at once do not produce noisy or divide-by-near-zero rates.
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
    pub error_rate_percent: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Rates {
    requests_per_sec: f64,
    bytes_in_sec: f64,
    bytes_out_sec: f64,
}

#[derive(Debug, Clone, Copy)]
struct Sample {
    at: Instant,
    requests_total: u64,
    bytes_received: u64,
    bytes_sent: u64,
    rates: Rates,
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
        let rates = match *last {
            Some(prev) if now.duration_since(prev.at) < MIN_SAMPLE_INTERVAL => prev.rates,
            Some(prev) => compute_rates(&prev, snapshot, now),
            None => Rates::default(),
        };
        let keep_previous =
            matches!(*last, Some(prev) if now.duration_since(prev.at) < MIN_SAMPLE_INTERVAL);
        if !keep_previous {
            *last = Some(Sample {
                at: now,
                requests_total: snapshot.requests_total,
                bytes_received: snapshot.bytes_received,
                bytes_sent: snapshot.bytes_sent,
                rates,
            });
        }
        build(snapshot, rates, process_memory_mb())
    }
}

fn compute_rates(prev: &Sample, snapshot: &MetricsSnapshot, now: Instant) -> Rates {
    let secs = now.duration_since(prev.at).as_secs_f64();
    if secs <= 0.0 {
        return prev.rates;
    }
    // Counters reset to zero when metrics are reset; treat that as no traffic.
    let per_sec = |current: u64, before: u64| current.saturating_sub(before) as f64 / secs;
    Rates {
        requests_per_sec: per_sec(snapshot.requests_total, prev.requests_total),
        bytes_in_sec: per_sec(snapshot.bytes_received, prev.bytes_received),
        bytes_out_sec: per_sec(snapshot.bytes_sent, prev.bytes_sent),
    }
}

fn build(snapshot: &MetricsSnapshot, rates: Rates, memory_usage_mb: f64) -> DashboardMetrics {
    let error_rate_percent = if snapshot.requests_total == 0 {
        0.0
    } else {
        snapshot.requests_error as f64 / snapshot.requests_total as f64 * 100.0
    };
    DashboardMetrics {
        timestamp: time::OffsetDateTime::now_utc().unix_timestamp() * 1000,
        requests_per_sec: rates.requests_per_sec,
        avg_response_time_ms: snapshot.avg_request_duration_us as f64 / 1000.0,
        memory_usage_mb,
        active_connections: snapshot.connections_active,
        bytes_in_sec: rates.bytes_in_sec,
        bytes_out_sec: rates.bytes_out_sec,
        error_rate_percent,
        uptime_seconds: snapshot.uptime_secs,
    }
}

/// Resident set size of the current process in MiB (Linux only).
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

    fn snapshot(requests: u64, errors: u64, received: u64, sent: u64) -> MetricsSnapshot {
        let mut s = interceptor_core::metrics::Metrics::new().snapshot();
        s.requests_total = requests;
        s.requests_error = errors;
        s.bytes_received = received;
        s.bytes_sent = sent;
        s
    }

    #[test]
    fn first_sample_has_zero_rates() {
        let sampler = RateSampler::new();
        let m = sampler.sample(&snapshot(100, 0, 1000, 2000), Instant::now());
        assert_eq!(m.requests_per_sec, 0.0);
        assert_eq!(m.bytes_in_sec, 0.0);
        assert_eq!(m.bytes_out_sec, 0.0);
    }

    #[test]
    fn rates_come_from_the_delta_between_samples() {
        let sampler = RateSampler::new();
        let t0 = Instant::now();
        sampler.sample(&snapshot(100, 0, 1000, 2000), t0);
        let m = sampler.sample(&snapshot(120, 0, 3000, 2500), t0 + Duration::from_secs(2));
        assert_eq!(m.requests_per_sec, 10.0);
        assert_eq!(m.bytes_in_sec, 1000.0);
        assert_eq!(m.bytes_out_sec, 250.0);
    }

    #[test]
    fn counter_reset_is_not_negative_traffic() {
        let sampler = RateSampler::new();
        let t0 = Instant::now();
        sampler.sample(&snapshot(500, 0, 9000, 9000), t0);
        let m = sampler.sample(&snapshot(3, 0, 10, 10), t0 + Duration::from_secs(1));
        assert_eq!(m.requests_per_sec, 0.0);
        assert_eq!(m.bytes_in_sec, 0.0);
    }

    #[test]
    fn fast_polls_reuse_the_previous_rates() {
        let sampler = RateSampler::new();
        let t0 = Instant::now();
        sampler.sample(&snapshot(0, 0, 0, 0), t0);
        let first = sampler.sample(&snapshot(10, 0, 0, 0), t0 + Duration::from_secs(1));
        let second = sampler.sample(&snapshot(11, 0, 0, 0), t0 + Duration::from_millis(1050));
        assert_eq!(first.requests_per_sec, 10.0);
        assert_eq!(second.requests_per_sec, 10.0);
    }

    #[test]
    fn error_rate_is_a_percentage_and_safe_when_idle() {
        let idle = build(&snapshot(0, 0, 0, 0), Rates::default(), 0.0);
        assert_eq!(idle.error_rate_percent, 0.0);
        let busy = build(&snapshot(200, 50, 0, 0), Rates::default(), 0.0);
        assert_eq!(busy.error_rate_percent, 25.0);
    }
}
