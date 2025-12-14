//! Security utilities for INT3RCEPTOR
//!
//! Provides comprehensive security features:
//! - Rate limiting (token bucket)
//! - IP allowlist/blocklist filtering
//! - CSRF token protection
//! - Audit logging
//! - Input validation
//! - Security headers
//! - TLS configuration

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::IpAddr;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
            global_bucket: RwLock::new(TokenBucket::new(max_requests.saturating_mul(10), refill_rate)),
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
    Allowed { remaining: u32 },
    Limited { remaining: u32, retry_after: Duration },
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
// IP FILTERING (ALLOWLIST / BLOCKLIST)
// ============================================================================

/// IP filter with allowlist and blocklist support
#[derive(Debug)]
pub struct IpFilter {
    /// If not empty, only these IPs are allowed
    allowlist: RwLock<HashSet<IpAddr>>,
    /// These IPs are always blocked (checked first)
    blocklist: RwLock<HashSet<IpAddr>>,
    /// CIDR ranges for allowlist (stored as (network, prefix_len))
    allowlist_ranges: RwLock<Vec<(IpAddr, u8)>>,
    /// CIDR ranges for blocklist
    blocklist_ranges: RwLock<Vec<(IpAddr, u8)>>,
    /// Enable/disable filtering
    enabled: RwLock<bool>,
    /// Mode: true = allowlist mode (deny by default), false = blocklist mode (allow by default)
    allowlist_mode: RwLock<bool>,
}

impl IpFilter {
    pub fn new() -> Self {
        Self {
            allowlist: RwLock::new(HashSet::new()),
            blocklist: RwLock::new(HashSet::new()),
            allowlist_ranges: RwLock::new(Vec::new()),
            blocklist_ranges: RwLock::new(Vec::new()),
            enabled: RwLock::new(false),
            allowlist_mode: RwLock::new(false),
        }
    }

    /// Enable or disable filtering
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.write() = enabled;
    }

    /// Set allowlist mode (true) or blocklist mode (false)
    pub fn set_allowlist_mode(&self, mode: bool) {
        *self.allowlist_mode.write() = mode;
    }

    /// Add IP to allowlist
    pub fn allow_ip(&self, ip: IpAddr) {
        self.allowlist.write().insert(ip);
    }

    /// Add IP to blocklist
    pub fn block_ip(&self, ip: IpAddr) {
        self.blocklist.write().insert(ip);
    }

    /// Remove IP from allowlist
    pub fn remove_from_allowlist(&self, ip: &IpAddr) {
        self.allowlist.write().remove(ip);
    }

    /// Remove IP from blocklist
    pub fn remove_from_blocklist(&self, ip: &IpAddr) {
        self.blocklist.write().remove(ip);
    }

    /// Add CIDR range to allowlist (e.g., "192.168.1.0/24")
    pub fn allow_cidr(&self, network: IpAddr, prefix_len: u8) {
        self.allowlist_ranges.write().push((network, prefix_len));
    }

    /// Add CIDR range to blocklist
    pub fn block_cidr(&self, network: IpAddr, prefix_len: u8) {
        self.blocklist_ranges.write().push((network, prefix_len));
    }

    /// Check if an IP is allowed
    pub fn is_allowed(&self, ip: IpAddr) -> IpFilterResult {
        if !*self.enabled.read() {
            return IpFilterResult::Allowed;
        }

        // Blocklist always takes precedence
        if self.is_in_blocklist(ip) {
            return IpFilterResult::Blocked {
                reason: "IP is in blocklist",
            };
        }

        let allowlist_mode = *self.allowlist_mode.read();

        if allowlist_mode {
            // Allowlist mode: must be in allowlist
            if self.is_in_allowlist(ip) {
                IpFilterResult::Allowed
            } else {
                IpFilterResult::Blocked {
                    reason: "IP is not in allowlist",
                }
            }
        } else {
            // Blocklist mode: allowed unless in blocklist (already checked)
            IpFilterResult::Allowed
        }
    }

    fn is_in_blocklist(&self, ip: IpAddr) -> bool {
        if self.blocklist.read().contains(&ip) {
            return true;
        }

        for (network, prefix_len) in self.blocklist_ranges.read().iter() {
            if ip_in_cidr(ip, *network, *prefix_len) {
                return true;
            }
        }

        false
    }

    fn is_in_allowlist(&self, ip: IpAddr) -> bool {
        let allowlist = self.allowlist.read();
        if allowlist.is_empty() && self.allowlist_ranges.read().is_empty() {
            // No allowlist configured = allow all
            return true;
        }

        if allowlist.contains(&ip) {
            return true;
        }

        for (network, prefix_len) in self.allowlist_ranges.read().iter() {
            if ip_in_cidr(ip, *network, *prefix_len) {
                return true;
            }
        }

        false
    }

    /// Clear all lists
    pub fn clear(&self) {
        self.allowlist.write().clear();
        self.blocklist.write().clear();
        self.allowlist_ranges.write().clear();
        self.blocklist_ranges.write().clear();
    }

    /// Get current configuration
    pub fn config(&self) -> IpFilterConfig {
        IpFilterConfig {
            enabled: *self.enabled.read(),
            allowlist_mode: *self.allowlist_mode.read(),
            allowlist_count: self.allowlist.read().len(),
            blocklist_count: self.blocklist.read().len(),
            allowlist_ranges_count: self.allowlist_ranges.read().len(),
            blocklist_ranges_count: self.blocklist_ranges.read().len(),
        }
    }
}

impl Default for IpFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if IP is in CIDR range
fn ip_in_cidr(ip: IpAddr, network: IpAddr, prefix_len: u8) -> bool {
    match (ip, network) {
        (IpAddr::V4(ip), IpAddr::V4(net)) => {
            if prefix_len > 32 {
                return false;
            }
            let mask = if prefix_len == 0 {
                0u32
            } else {
                !0u32 << (32 - prefix_len)
            };
            (u32::from(ip) & mask) == (u32::from(net) & mask)
        }
        (IpAddr::V6(ip), IpAddr::V6(net)) => {
            if prefix_len > 128 {
                return false;
            }
            let ip_bits = u128::from(ip);
            let net_bits = u128::from(net);
            let mask = if prefix_len == 0 {
                0u128
            } else {
                !0u128 << (128 - prefix_len)
            };
            (ip_bits & mask) == (net_bits & mask)
        }
        _ => false, // IPv4 vs IPv6 mismatch
    }
}

#[derive(Debug, Clone)]
pub enum IpFilterResult {
    Allowed,
    Blocked { reason: &'static str },
}

impl IpFilterResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, IpFilterResult::Allowed)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IpFilterConfig {
    pub enabled: bool,
    pub allowlist_mode: bool,
    pub allowlist_count: usize,
    pub blocklist_count: usize,
    pub allowlist_ranges_count: usize,
    pub blocklist_ranges_count: usize,
}

// ============================================================================
// CSRF PROTECTION
// ============================================================================

/// CSRF token manager
#[derive(Debug)]
pub struct CsrfManager {
    /// Token to session mapping with expiry
    tokens: RwLock<HashMap<String, CsrfToken>>,
    /// Token lifetime
    token_lifetime: Duration,
    /// Counter for token generation
    counter: AtomicU64,
    /// Secret key for token generation
    secret: [u8; 32],
}

#[derive(Debug, Clone)]
struct CsrfToken {
    session_id: String,
    created_at: Instant,
}

impl CsrfManager {
    /// Create a new CSRF manager with a random secret
    pub fn new(token_lifetime: Duration) -> Self {
        let mut secret = [0u8; 32];
        // Use system randomness
        if let Ok(bytes) = std::fs::read("/dev/urandom") {
            for (i, &b) in bytes.iter().take(32).enumerate() {
                secret[i] = b;
            }
        } else {
            // Fallback: use time-based seed
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let seed = now.as_nanos() as u64;
            for (i, chunk) in secret.chunks_mut(8).enumerate() {
                let val = seed.wrapping_mul(i as u64 + 1);
                chunk.copy_from_slice(&val.to_le_bytes()[..chunk.len()]);
            }
        }

        Self {
            tokens: RwLock::new(HashMap::new()),
            token_lifetime,
            counter: AtomicU64::new(1),
            secret,
        }
    }

    /// Generate a new CSRF token for a session
    pub fn generate_token(&self, session_id: &str) -> String {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Create token: hash of (secret + counter + timestamp + session_id)
        let mut data = Vec::with_capacity(64);
        data.extend_from_slice(&self.secret);
        data.extend_from_slice(&counter.to_le_bytes());
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(session_id.as_bytes());

        let token = simple_hash(&data);

        // Store token
        self.tokens.write().insert(
            token.clone(),
            CsrfToken {
                session_id: session_id.to_string(),
                created_at: Instant::now(),
            },
        );

        token
    }

    /// Validate a CSRF token
    pub fn validate_token(&self, token: &str, session_id: &str) -> CsrfValidationResult {
        let tokens = self.tokens.read();

        match tokens.get(token) {
            None => CsrfValidationResult::Invalid {
                reason: "Token not found",
            },
            Some(stored) => {
                // Check session match
                if !constant_time_compare(&stored.session_id, session_id) {
                    return CsrfValidationResult::Invalid {
                        reason: "Session mismatch",
                    };
                }

                // Check expiry
                if stored.created_at.elapsed() > self.token_lifetime {
                    return CsrfValidationResult::Expired;
                }

                CsrfValidationResult::Valid
            }
        }
    }

    /// Consume (invalidate) a token after use (for single-use tokens)
    pub fn consume_token(&self, token: &str, session_id: &str) -> CsrfValidationResult {
        let result = self.validate_token(token, session_id);
        if matches!(result, CsrfValidationResult::Valid) {
            self.tokens.write().remove(token);
        }
        result
    }

    /// Clean up expired tokens
    pub fn cleanup(&self) {
        let mut tokens = self.tokens.write();
        tokens.retain(|_, t| t.created_at.elapsed() < self.token_lifetime);
    }

    /// Get stats
    pub fn stats(&self) -> CsrfStats {
        CsrfStats {
            active_tokens: self.tokens.read().len(),
            token_lifetime_secs: self.token_lifetime.as_secs(),
        }
    }
}

impl Default for CsrfManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(3600)) // 1 hour default
    }
}

/// Simple hash function for token generation (not cryptographic, but fast)
fn simple_hash(data: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }

    // Add more mixing
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    hash ^= timestamp;
    hash = hash.wrapping_mul(0x517cc1b727220a95);

    format!("{:016x}{:016x}", hash, hash.rotate_right(32) ^ timestamp)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CsrfValidationResult {
    Valid,
    Invalid { reason: &'static str },
    Expired,
}

impl CsrfValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, CsrfValidationResult::Valid)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CsrfStats {
    pub active_tokens: usize,
    pub token_lifetime_secs: u64,
}

// ============================================================================
// AUDIT LOGGING
// ============================================================================

/// Security audit logger
#[derive(Debug)]
pub struct AuditLogger {
    /// In-memory log buffer
    logs: RwLock<VecDeque<AuditEntry>>,
    /// Maximum entries to keep in memory
    max_entries: usize,
    /// File path for persistent logging
    file_path: Option<std::path::PathBuf>,
    /// Counter for entry IDs
    counter: AtomicU64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub source_ip: Option<String>,
    pub user_id: Option<String>,
    pub action: String,
    pub resource: Option<String>,
    pub outcome: AuditOutcome,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    Configuration,
    Security,
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure,
    Blocked,
    Error,
}

impl AuditLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            logs: RwLock::new(VecDeque::with_capacity(max_entries)),
            max_entries,
            file_path: None,
            counter: AtomicU64::new(1),
        }
    }

    /// Set file path for persistent logging
    pub fn with_file(mut self, path: impl AsRef<Path>) -> Self {
        self.file_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Log an audit event
    pub fn log(&self, entry: AuditEntryBuilder) -> u64 {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = AuditEntry {
            id,
            timestamp,
            event_type: entry.event_type,
            severity: entry.severity,
            source_ip: entry.source_ip,
            user_id: entry.user_id,
            action: entry.action,
            resource: entry.resource,
            outcome: entry.outcome,
            details: entry.details,
        };

        // Log to tracing
        match entry.severity {
            AuditSeverity::Debug => tracing::debug!(
                audit_id = id,
                event = ?entry.event_type,
                action = %entry.action,
                outcome = ?entry.outcome,
                "Audit event"
            ),
            AuditSeverity::Info => tracing::info!(
                audit_id = id,
                event = ?entry.event_type,
                action = %entry.action,
                outcome = ?entry.outcome,
                "Audit event"
            ),
            AuditSeverity::Warning => tracing::warn!(
                audit_id = id,
                event = ?entry.event_type,
                action = %entry.action,
                outcome = ?entry.outcome,
                "Audit event"
            ),
            AuditSeverity::Error | AuditSeverity::Critical => tracing::error!(
                audit_id = id,
                event = ?entry.event_type,
                action = %entry.action,
                outcome = ?entry.outcome,
                "Audit event"
            ),
        }

        // Store in memory
        let mut logs = self.logs.write();
        if logs.len() >= self.max_entries {
            logs.pop_front();
        }
        logs.push_back(entry.clone());

        // Write to file if configured
        if let Some(ref path) = self.file_path {
            if let Ok(json) = serde_json::to_string(&entry) {
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .and_then(|mut f| {
                        use std::io::Write;
                        writeln!(f, "{}", json)
                    });
            }
        }

        id
    }

    /// Get recent audit entries
    pub fn get_entries(&self, limit: usize) -> Vec<AuditEntry> {
        let logs = self.logs.read();
        logs.iter().rev().take(limit).cloned().collect()
    }

    /// Get entries by severity
    pub fn get_by_severity(&self, min_severity: AuditSeverity, limit: usize) -> Vec<AuditEntry> {
        let logs = self.logs.read();
        logs.iter()
            .rev()
            .filter(|e| e.severity >= min_severity)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get entries by event type
    pub fn get_by_type(&self, event_type: AuditEventType, limit: usize) -> Vec<AuditEntry> {
        let logs = self.logs.read();
        logs.iter()
            .rev()
            .filter(|e| e.event_type == event_type)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Search entries
    pub fn search(&self, query: &str, limit: usize) -> Vec<AuditEntry> {
        let query_lower = query.to_lowercase();
        let logs = self.logs.read();
        logs.iter()
            .rev()
            .filter(|e| {
                e.action.to_lowercase().contains(&query_lower)
                    || e.resource
                        .as_ref()
                        .map(|r| r.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || e.details
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.logs.write().clear();
    }

    /// Helper methods for common audit events
    pub fn log_auth_success(&self, source_ip: Option<&str>, user_id: &str) -> u64 {
        self.log(
            AuditEntryBuilder::new(AuditEventType::Authentication, "login")
                .severity(AuditSeverity::Info)
                .source_ip(source_ip)
                .user_id(Some(user_id))
                .outcome(AuditOutcome::Success),
        )
    }

    pub fn log_auth_failure(&self, source_ip: Option<&str>, reason: &str) -> u64 {
        self.log(
            AuditEntryBuilder::new(AuditEventType::Authentication, "login")
                .severity(AuditSeverity::Warning)
                .source_ip(source_ip)
                .outcome(AuditOutcome::Failure)
                .details(Some(reason)),
        )
    }

    pub fn log_access_blocked(&self, source_ip: Option<&str>, resource: &str, reason: &str) -> u64 {
        self.log(
            AuditEntryBuilder::new(AuditEventType::Security, "access_blocked")
                .severity(AuditSeverity::Warning)
                .source_ip(source_ip)
                .resource(Some(resource))
                .outcome(AuditOutcome::Blocked)
                .details(Some(reason)),
        )
    }

    pub fn log_rate_limited(&self, source_ip: Option<&str>) -> u64 {
        self.log(
            AuditEntryBuilder::new(AuditEventType::Security, "rate_limited")
                .severity(AuditSeverity::Warning)
                .source_ip(source_ip)
                .outcome(AuditOutcome::Blocked)
                .details(Some("Rate limit exceeded")),
        )
    }

    pub fn log_config_change(&self, user_id: Option<&str>, what: &str, details: &str) -> u64 {
        self.log(
            AuditEntryBuilder::new(AuditEventType::Configuration, "config_change")
                .severity(AuditSeverity::Info)
                .user_id(user_id)
                .resource(Some(what))
                .outcome(AuditOutcome::Success)
                .details(Some(details)),
        )
    }

    pub fn log_security_event(&self, source_ip: Option<&str>, event: &str, severity: AuditSeverity) -> u64 {
        self.log(
            AuditEntryBuilder::new(AuditEventType::Security, event)
                .severity(severity)
                .source_ip(source_ip)
                .outcome(AuditOutcome::Blocked),
        )
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000)
    }
}

/// Builder for audit entries
pub struct AuditEntryBuilder {
    event_type: AuditEventType,
    severity: AuditSeverity,
    source_ip: Option<String>,
    user_id: Option<String>,
    action: String,
    resource: Option<String>,
    outcome: AuditOutcome,
    details: Option<String>,
}

impl AuditEntryBuilder {
    pub fn new(event_type: AuditEventType, action: impl Into<String>) -> Self {
        Self {
            event_type,
            severity: AuditSeverity::Info,
            source_ip: None,
            user_id: None,
            action: action.into(),
            resource: None,
            outcome: AuditOutcome::Success,
            details: None,
        }
    }

    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn source_ip(mut self, ip: Option<&str>) -> Self {
        self.source_ip = ip.map(String::from);
        self
    }

    pub fn user_id(mut self, id: Option<&str>) -> Self {
        self.user_id = id.map(String::from);
        self
    }

    pub fn resource(mut self, resource: Option<&str>) -> Self {
        self.resource = resource.map(String::from);
        self
    }

    pub fn outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub fn details(mut self, details: Option<&str>) -> Self {
        self.details = details.map(String::from);
        self
    }
}

// ============================================================================
// TLS CONFIGURATION
// ============================================================================

/// TLS configuration for API server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,
    /// Path to certificate file (PEM)
    pub cert_path: Option<String>,
    /// Path to private key file (PEM)
    pub key_path: Option<String>,
    /// Minimum TLS version (1.2 or 1.3)
    pub min_version: TlsVersion,
    /// Require client certificates
    pub require_client_cert: bool,
    /// Path to CA certificate for client verification
    pub client_ca_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TlsVersion {
    #[serde(rename = "1.2")]
    Tls12,
    #[serde(rename = "1.3")]
    Tls13,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            min_version: TlsVersion::Tls12,
            require_client_cert: false,
            client_ca_path: None,
        }
    }
}

impl TlsConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.enabled {
            return Ok(());
        }

        if self.cert_path.is_none() {
            return Err("TLS enabled but no certificate path specified");
        }

        if self.key_path.is_none() {
            return Err("TLS enabled but no key path specified");
        }

        if let Some(ref path) = self.cert_path {
            if !Path::new(path).exists() {
                return Err("Certificate file does not exist");
            }
        }

        if let Some(ref path) = self.key_path {
            if !Path::new(path).exists() {
                return Err("Key file does not exist");
            }
        }

        if self.require_client_cert && self.client_ca_path.is_none() {
            return Err("Client cert required but no CA path specified");
        }

        Ok(())
    }
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

/// Timing-safe byte comparison
pub fn constant_time_compare_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// ============================================================================
// INPUT VALIDATION
// ============================================================================

pub mod validation {
    use std::net::IpAddr;

    pub const MAX_URL_LENGTH: usize = 8192;
    pub const MAX_HEADER_LENGTH: usize = 8192;
    pub const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

    pub fn validate_url(url: &str) -> Result<(), ValidationError> {
        if url.is_empty() {
            return Err(ValidationError::Empty("URL"));
        }
        if url.len() > MAX_URL_LENGTH {
            return Err(ValidationError::TooLong("URL", MAX_URL_LENGTH));
        }
        if url.contains('\0') {
            return Err(ValidationError::InvalidCharacter("URL", "null byte"));
        }
        Ok(())
    }

    pub fn validate_header_name(name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::Empty("header name"));
        }
        if name.len() > 256 {
            return Err(ValidationError::TooLong("header name", 256));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii() && c != ':' && !c.is_control())
        {
            return Err(ValidationError::InvalidCharacter(
                "header name",
                "non-ASCII or control character",
            ));
        }
        Ok(())
    }

    pub fn validate_header_value(value: &str) -> Result<(), ValidationError> {
        if value.len() > MAX_HEADER_LENGTH {
            return Err(ValidationError::TooLong("header value", MAX_HEADER_LENGTH));
        }
        if value.contains('\r') || value.contains('\n') {
            return Err(ValidationError::InvalidCharacter(
                "header value",
                "CRLF characters",
            ));
        }
        Ok(())
    }

    pub fn validate_body_size(size: usize, max: usize) -> Result<(), ValidationError> {
        if size > max {
            return Err(ValidationError::TooLarge("body", max));
        }
        Ok(())
    }

    pub fn validate_ip(ip: &str) -> Result<IpAddr, ValidationError> {
        ip.parse()
            .map_err(|_| ValidationError::InvalidFormat("IP address"))
    }

    pub fn validate_port(port: u16) -> Result<(), ValidationError> {
        if port == 0 {
            return Err(ValidationError::InvalidValue("port", "cannot be 0"));
        }
        Ok(())
    }

    pub fn sanitize_for_log(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_control() { '?' } else { c })
            .take(1000)
            .collect()
    }

    #[derive(Debug, Clone)]
    pub enum ValidationError {
        Empty(&'static str),
        TooLong(&'static str, usize),
        TooLarge(&'static str, usize),
        InvalidCharacter(&'static str, &'static str),
        InvalidFormat(&'static str),
        InvalidValue(&'static str, &'static str),
    }

    impl std::fmt::Display for ValidationError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Empty(field) => write!(f, "{} cannot be empty", field),
                Self::TooLong(field, max) => {
                    write!(f, "{} exceeds maximum length of {}", field, max)
                }
                Self::TooLarge(field, max) => {
                    write!(f, "{} exceeds maximum size of {} bytes", field, max)
                }
                Self::InvalidCharacter(field, chars) => {
                    write!(f, "{} contains invalid {}", field, chars)
                }
                Self::InvalidFormat(field) => write!(f, "invalid {} format", field),
                Self::InvalidValue(field, reason) => write!(f, "invalid {}: {}", field, reason),
            }
        }
    }

    impl std::error::Error for ValidationError {}
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
    pub const REFERRER_POLICY: (&str, &str) = ("Referrer-Policy", "strict-origin-when-cross-origin");
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
    fn test_ip_filter_blocklist() {
        let filter = IpFilter::new();
        filter.set_enabled(true);
        filter.block_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));

        let blocked = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let allowed = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 101));

        assert!(!filter.is_allowed(blocked).is_allowed());
        assert!(filter.is_allowed(allowed).is_allowed());
    }

    #[test]
    fn test_ip_filter_allowlist() {
        let filter = IpFilter::new();
        filter.set_enabled(true);
        filter.set_allowlist_mode(true);
        filter.allow_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));

        let allowed = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let blocked = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

        assert!(filter.is_allowed(allowed).is_allowed());
        assert!(!filter.is_allowed(blocked).is_allowed());
    }

    #[test]
    fn test_ip_filter_cidr() {
        let filter = IpFilter::new();
        filter.set_enabled(true);
        filter.block_cidr(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)), 16);

        let blocked1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let blocked2 = IpAddr::V4(Ipv4Addr::new(192, 168, 255, 255));
        let allowed = IpAddr::V4(Ipv4Addr::new(192, 169, 0, 1));

        assert!(!filter.is_allowed(blocked1).is_allowed());
        assert!(!filter.is_allowed(blocked2).is_allowed());
        assert!(filter.is_allowed(allowed).is_allowed());
    }

    #[test]
    fn test_csrf_token_lifecycle() {
        let manager = CsrfManager::new(Duration::from_secs(3600));
        let session = "session123";

        let token = manager.generate_token(session);
        assert!(!token.is_empty());

        // Valid token
        assert!(manager.validate_token(&token, session).is_valid());

        // Wrong session
        assert!(!manager.validate_token(&token, "wrong_session").is_valid());

        // Wrong token
        assert!(!manager.validate_token("invalid_token", session).is_valid());
    }

    #[test]
    fn test_csrf_token_consumption() {
        let manager = CsrfManager::new(Duration::from_secs(3600));
        let session = "session456";

        let token = manager.generate_token(session);

        // First use: valid
        assert!(manager.consume_token(&token, session).is_valid());

        // Second use: invalid (consumed)
        assert!(!manager.consume_token(&token, session).is_valid());
    }

    #[test]
    fn test_audit_logger() {
        let logger = AuditLogger::new(100);

        let id = logger.log_auth_success(Some("192.168.1.1"), "user123");
        assert!(id > 0);

        let entries = logger.get_entries(10);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].outcome, AuditOutcome::Success);
    }

    #[test]
    fn test_audit_logger_search() {
        let logger = AuditLogger::new(100);

        logger.log_auth_success(Some("192.168.1.1"), "admin");
        logger.log_auth_failure(Some("10.0.0.1"), "invalid password");
        logger.log_access_blocked(Some("1.2.3.4"), "/admin", "unauthorized");

        let results = logger.search("admin", 10);
        assert_eq!(results.len(), 2); // admin user + /admin resource
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("secret123", "secret123"));
        assert!(!constant_time_compare("secret123", "secret124"));
        assert!(!constant_time_compare("short", "longer_string"));
        assert!(constant_time_compare("", ""));
    }

    #[test]
    fn test_constant_time_compare_bytes() {
        assert!(constant_time_compare_bytes(b"secret", b"secret"));
        assert!(!constant_time_compare_bytes(b"secret", b"secreT"));
        assert!(!constant_time_compare_bytes(b"short", b"longer"));
    }

    #[test]
    fn test_tls_config_validation() {
        let mut config = TlsConfig::default();
        assert!(config.validate().is_ok()); // Disabled = OK

        config.enabled = true;
        assert!(config.validate().is_err()); // No cert

        config.cert_path = Some("/nonexistent".into());
        config.key_path = Some("/nonexistent".into());
        assert!(config.validate().is_err()); // Files don't exist
    }

    #[test]
    fn test_ip_in_cidr() {
        // IPv4 tests
        let network = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0));
        assert!(ip_in_cidr(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            network,
            16
        ));
        assert!(ip_in_cidr(
            IpAddr::V4(Ipv4Addr::new(192, 168, 255, 255)),
            network,
            16
        ));
        assert!(!ip_in_cidr(
            IpAddr::V4(Ipv4Addr::new(192, 169, 0, 1)),
            network,
            16
        ));

        // /24 subnet
        let network24 = IpAddr::V4(Ipv4Addr::new(10, 0, 1, 0));
        assert!(ip_in_cidr(
            IpAddr::V4(Ipv4Addr::new(10, 0, 1, 100)),
            network24,
            24
        ));
        assert!(!ip_in_cidr(
            IpAddr::V4(Ipv4Addr::new(10, 0, 2, 1)),
            network24,
            24
        ));
    }

    #[test]
    fn test_validate_url() {
        use validation::*;
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("").is_err());
        assert!(validate_url("https://example.com/path\0evil").is_err());
        let long_url = "a".repeat(MAX_URL_LENGTH + 1);
        assert!(validate_url(&long_url).is_err());
    }

    #[test]
    fn test_validate_header_name() {
        use validation::*;
        assert!(validate_header_name("Content-Type").is_ok());
        assert!(validate_header_name("X-Custom-Header").is_ok());
        assert!(validate_header_name("").is_err());
        assert!(validate_header_name("Header:Name").is_err());
        assert!(validate_header_name("Header\nName").is_err());
    }

    #[test]
    fn test_validate_header_value() {
        use validation::*;
        assert!(validate_header_value("application/json").is_ok());
        assert!(validate_header_value("value\r\nHeader: injection").is_err());
    }

    #[test]
    fn test_sanitize_for_log() {
        use validation::*;
        assert_eq!(sanitize_for_log("normal text"), "normal text");
        assert_eq!(sanitize_for_log("with\nnewline"), "with?newline");
        assert_eq!(sanitize_for_log("with\ttab"), "with?tab");
    }

    #[test]
    fn test_security_headers() {
        let hdrs = headers::all();
        assert!(hdrs.len() >= 6);
        assert!(hdrs.iter().any(|(k, _)| *k == "X-Content-Type-Options"));
        assert!(hdrs
            .iter()
            .any(|(k, _)| *k == "Strict-Transport-Security"));
    }
}
