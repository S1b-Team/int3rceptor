//! Security utilities shared by the API:
//! - Rate limiting (token bucket)
//! - Timing-safe string comparison
//! - Security headers

use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

// ============================================================================
// RATE LIMITING
// ============================================================================

/// Rate limiter using token bucket algorithm
#[derive(Debug)]
pub struct RateLimiter {
    max_requests: u32,
    window: Duration,
    buckets: RwLock<HashMap<IpAddr, TokenBucket>>,
    global_bucket: RwLock<TokenBucket>,
}

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: u32,
    max_tokens: u32,
    last_refill: Instant,
    refill_rate: Duration,
}

impl TokenBucket {
    fn new(max_tokens: u32, refill_rate: Duration) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            last_refill: Instant::now(),
            refill_rate,
        }
    }

    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        if self.refill_rate.as_millis() > 0 {
            let refills = (elapsed.as_millis() / self.refill_rate.as_millis()) as u32;
            if refills > 0 {
                self.tokens = (self.tokens + refills).min(self.max_tokens);
                self.last_refill = now;
            }
        }
    }

    fn remaining(&self) -> u32 {
        self.tokens
    }
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        let refill_rate = if max_requests > 0 {
            Duration::from_millis(window.as_millis() as u64 / max_requests as u64)
        } else {
            Duration::from_secs(1)
        };
        Self {
            max_requests,
            window,
            buckets: RwLock::new(HashMap::new()),
            global_bucket: RwLock::new(TokenBucket::new(
                max_requests.saturating_mul(10),
                refill_rate,
            )),
        }
    }

    pub fn check(&self, ip: IpAddr) -> RateLimitResult {
        {
            let mut global = self.global_bucket.write();
            if !global.try_consume() {
                return RateLimitResult::Limited {
                    remaining: 0,
                    retry_after: self.window,
                };
            }
        }

        let mut buckets = self.buckets.write();
        let bucket = buckets.entry(ip).or_insert_with(|| {
            let refill_rate = if self.max_requests > 0 {
                Duration::from_millis(self.window.as_millis() as u64 / self.max_requests as u64)
            } else {
                Duration::from_secs(1)
            };
            TokenBucket::new(self.max_requests, refill_rate)
        });

        if bucket.try_consume() {
            RateLimitResult::Allowed {
                remaining: bucket.remaining(),
            }
        } else {
            RateLimitResult::Limited {
                remaining: 0,
                retry_after: bucket.refill_rate,
            }
        }
    }

    pub fn cleanup(&self, max_age: Duration) {
        let mut buckets = self.buckets.write();
        let now = Instant::now();
        buckets.retain(|_, bucket| now.duration_since(bucket.last_refill) < max_age);
    }

    pub fn stats(&self) -> RateLimiterStats {
        let buckets = self.buckets.read();
        RateLimiterStats {
            tracked_ips: buckets.len(),
            max_requests: self.max_requests,
            window_secs: self.window.as_secs(),
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(100, Duration::from_secs(60))
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed {
        remaining: u32,
    },
    Limited {
        remaining: u32,
        retry_after: Duration,
    },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RateLimiterStats {
    pub tracked_ips: usize,
    pub max_requests: u32,
    pub window_secs: u64,
}

// ============================================================================
// TIMING-SAFE COMPARISON
// ============================================================================

/// Timing-safe string comparison to prevent timing attacks
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

// ============================================================================
// SECURITY HEADERS
// ============================================================================

pub mod headers {
    pub const X_CONTENT_TYPE_OPTIONS: (&str, &str) = ("X-Content-Type-Options", "nosniff");
    pub const X_FRAME_OPTIONS: (&str, &str) = ("X-Frame-Options", "DENY");
    pub const X_XSS_PROTECTION: (&str, &str) = ("X-XSS-Protection", "1; mode=block");
    pub const CACHE_CONTROL: (&str, &str) =
        ("Cache-Control", "no-store, no-cache, must-revalidate");
    pub const PRAGMA: (&str, &str) = ("Pragma", "no-cache");
    pub const CSP: (&str, &str) = (
        "Content-Security-Policy",
        "default-src 'none'; frame-ancestors 'none'",
    );
    pub const STRICT_TRANSPORT_SECURITY: (&str, &str) = (
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains; preload",
    );
    pub const REFERRER_POLICY: (&str, &str) =
        ("Referrer-Policy", "strict-origin-when-cross-origin");
    pub const PERMISSIONS_POLICY: (&str, &str) = (
        "Permissions-Policy",
        "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()",
    );

    pub fn all() -> Vec<(&'static str, &'static str)> {
        vec![
            X_CONTENT_TYPE_OPTIONS,
            X_FRAME_OPTIONS,
            X_XSS_PROTECTION,
            CACHE_CONTROL,
            PRAGMA,
            CSP,
            STRICT_TRANSPORT_SECURITY,
            REFERRER_POLICY,
            PERMISSIONS_POLICY,
        ]
    }

    /// Headers for non-TLS responses (excludes HSTS)
    pub fn all_no_hsts() -> Vec<(&'static str, &'static str)> {
        vec![
            X_CONTENT_TYPE_OPTIONS,
            X_FRAME_OPTIONS,
            X_XSS_PROTECTION,
            CACHE_CONTROL,
            PRAGMA,
            CSP,
            REFERRER_POLICY,
            PERMISSIONS_POLICY,
        ]
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        for _ in 0..10 {
            assert!(limiter.check(ip).is_allowed());
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        for _ in 0..5 {
            assert!(limiter.check(ip).is_allowed());
        }
        assert!(!limiter.check(ip).is_allowed());
    }

    #[test]
    fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        let ip1 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        assert!(limiter.check(ip1).is_allowed());
        assert!(limiter.check(ip1).is_allowed());
        assert!(!limiter.check(ip1).is_allowed());
        assert!(limiter.check(ip2).is_allowed());
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("secret123", "secret123"));
        assert!(!constant_time_compare("secret123", "secret124"));
        assert!(!constant_time_compare("short", "longer_string"));
        assert!(constant_time_compare("", ""));
    }

    #[test]
    fn test_security_headers() {
        let hdrs = headers::all();
        assert!(hdrs.len() >= 6);
        assert!(hdrs.iter().any(|(k, _)| *k == "X-Content-Type-Options"));
        assert!(hdrs.iter().any(|(k, _)| *k == "Strict-Transport-Security"));
    }
}
