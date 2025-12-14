//! Scanner Module - Vulnerability Detection Engine
//!
//! This module provides automated vulnerability scanning capabilities
//! including passive analysis and active testing.

mod detection;
mod rules;
mod types;

pub use detection::*;
pub use rules::*;
pub use types::*;

use crate::capture::CaptureEntry;
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Scanner Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanConfig {
    /// Enable passive scanning (analyze existing traffic)
    pub passive: bool,
    /// Enable active scanning (send test payloads)
    pub active: bool,
    /// Categories to scan for
    pub categories: Vec<VulnerabilityCategory>,
    /// Maximum concurrent active tests
    pub concurrency: usize,
    /// Delay between active tests (ms)
    pub delay_ms: u64,
    /// Follow redirects during active scanning
    pub follow_redirects: bool,
    /// Maximum request depth for crawling
    pub max_depth: u32,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            passive: true,
            active: false,
            categories: vec![
                VulnerabilityCategory::Injection,
                VulnerabilityCategory::XSS,
                VulnerabilityCategory::PathTraversal,
                VulnerabilityCategory::InformationDisclosure,
                VulnerabilityCategory::SecurityMisconfiguration,
            ],
            concurrency: 5,
            delay_ms: 100,
            follow_redirects: true,
            max_depth: 3,
        }
    }
}

/// Main Scanner struct
pub struct Scanner {
    config: RwLock<ScanConfig>,
    rules: RwLock<Vec<DetectionRule>>,
    findings: RwLock<Vec<Finding>>,
    is_running: AtomicBool,
    scan_id: AtomicU64,
    requests_scanned: AtomicU64,
    vulnerabilities_found: AtomicU64,
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(ScanConfig::default()),
            rules: RwLock::new(DetectionRule::default_rules()),
            findings: RwLock::new(Vec::new()),
            is_running: AtomicBool::new(false),
            scan_id: AtomicU64::new(0),
            requests_scanned: AtomicU64::new(0),
            vulnerabilities_found: AtomicU64::new(0),
        }
    }

    /// Update scanner configuration
    pub fn configure(&self, config: ScanConfig) {
        *self.config.write() = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> ScanConfig {
        self.config.read().clone()
    }

    /// Add a custom detection rule
    pub fn add_rule(&self, rule: DetectionRule) {
        self.rules.write().push(rule);
    }

    /// Get all detection rules
    pub fn get_rules(&self) -> Vec<DetectionRule> {
        self.rules.read().clone()
    }

    /// Perform passive scan on a captured entry
    pub fn passive_scan(&self, entry: &CaptureEntry) -> Vec<Finding> {
        let config = self.config.read();
        if !config.passive {
            return Vec::new();
        }

        let rules = self.rules.read();
        let mut findings = Vec::new();

        for rule in rules.iter() {
            if !config.categories.contains(&rule.category) {
                continue;
            }

            if let Some(finding) = rule.check_passive(entry) {
                findings.push(finding);
                self.vulnerabilities_found.fetch_add(1, Ordering::SeqCst);
            }
        }

        self.requests_scanned.fetch_add(1, Ordering::SeqCst);

        // Store findings
        if !findings.is_empty() {
            self.findings.write().extend(findings.clone());
        }

        findings
    }

    /// Start an active scan
    pub async fn start_active_scan(
        self: Arc<Self>,
        targets: Vec<String>,
        pool: crate::ConnectionPool,
    ) -> Result<u64> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Scan already running"));
        }

        let scan_id = self.scan_id.fetch_add(1, Ordering::SeqCst) + 1;
        let config = self.get_config();

        let scanner = self.clone();

        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(config.concurrency));

            for target in targets {
                if !scanner.is_running.load(Ordering::SeqCst) {
                    break;
                }

                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let scanner = scanner.clone();
                let pool = pool.clone();
                let delay = config.delay_ms;

                tokio::spawn(async move {
                    let _permit = permit;

                    // Run active tests
                    scanner.run_active_tests(&target, &pool).await;

                    if delay > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    }
                });
            }

            scanner.is_running.store(false, Ordering::SeqCst);
        });

        Ok(scan_id)
    }

    /// Run active tests against a target URL
    async fn run_active_tests(&self, target: &str, pool: &crate::ConnectionPool) {
        let config = self.config.read().clone();
        let rules = self.rules.read().clone();

        for rule in rules.iter() {
            if !config.categories.contains(&rule.category) {
                continue;
            }

            if !rule.active_payloads.is_empty() {
                for payload in &rule.active_payloads {
                    if !self.is_running.load(Ordering::SeqCst) {
                        return;
                    }

                    if let Some(finding) = self.test_payload(target, payload, &rule, pool).await {
                        self.findings.write().push(finding);
                        self.vulnerabilities_found.fetch_add(1, Ordering::SeqCst);
                    }

                    self.requests_scanned.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }

    /// Test a single payload against target
    async fn test_payload(
        &self,
        target: &str,
        payload: &str,
        rule: &DetectionRule,
        pool: &crate::ConnectionPool,
    ) -> Option<Finding> {
        // Inject payload into URL parameter
        let test_url = if target.contains('?') {
            format!("{}&test={}", target, urlencoding::encode(payload))
        } else {
            format!("{}?test={}", target, urlencoding::encode(payload))
        };

        // Build request
        let request = match http::Request::builder()
            .method("GET")
            .uri(&test_url)
            .header("User-Agent", "Int3rceptor-Scanner/1.0")
            .body(http_body_util::Full::new(bytes::Bytes::new()))
        {
            Ok(req) => req,
            Err(_) => return None,
        };

        // Send request
        let client = pool.client();
        let start = std::time::Instant::now();

        match client.request(request).await {
            Ok(response) => {
                let status = response.status().as_u16();
                let duration = start.elapsed().as_millis() as u64;

                // Check response for vulnerability indicators
                if let Ok(body) = http_body_util::BodyExt::collect(response.into_body()).await {
                    let body_bytes = body.to_bytes();
                    let body_str = String::from_utf8_lossy(&body_bytes);

                    // Check if payload is reflected (potential XSS)
                    if rule.category == VulnerabilityCategory::XSS && body_str.contains(payload) {
                        return Some(Finding {
                            id: uuid::Uuid::new_v4().to_string(),
                            rule_id: rule.id.clone(),
                            category: rule.category.clone(),
                            severity: rule.severity.clone(),
                            title: format!("{} - Payload Reflected", rule.name),
                            description: format!(
                                "The payload '{}' was reflected in the response",
                                payload
                            ),
                            url: test_url,
                            evidence: body_str[..body_str.len().min(500)].to_string(),
                            request: None,
                            response: Some(ResponseInfo {
                                status,
                                length: body_bytes.len(),
                                duration_ms: duration,
                            }),
                            remediation: rule.remediation.clone(),
                            references: rule.references.clone(),
                            timestamp: chrono::Utc::now(),
                            confirmed: false,
                        });
                    }

                    // Check for error-based detection (SQL injection)
                    if rule.category == VulnerabilityCategory::Injection {
                        for pattern in &rule.response_patterns {
                            if body_str.to_lowercase().contains(&pattern.to_lowercase()) {
                                return Some(Finding {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    rule_id: rule.id.clone(),
                                    category: rule.category.clone(),
                                    severity: rule.severity.clone(),
                                    title: format!("{} - Error Pattern Detected", rule.name),
                                    description: format!(
                                        "Database error pattern '{}' detected in response",
                                        pattern
                                    ),
                                    url: test_url,
                                    evidence: body_str[..body_str.len().min(500)].to_string(),
                                    request: None,
                                    response: Some(ResponseInfo {
                                        status,
                                        length: body_bytes.len(),
                                        duration_ms: duration,
                                    }),
                                    remediation: rule.remediation.clone(),
                                    references: rule.references.clone(),
                                    timestamp: chrono::Utc::now(),
                                    confirmed: true,
                                });
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }

        None
    }

    /// Stop active scanning
    pub fn stop_scan(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// Check if scan is running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Get all findings
    pub fn get_findings(&self) -> Vec<Finding> {
        self.findings.read().clone()
    }

    /// Get findings by severity
    pub fn get_findings_by_severity(&self, severity: Severity) -> Vec<Finding> {
        self.findings
            .read()
            .iter()
            .filter(|f| f.severity == severity)
            .cloned()
            .collect()
    }

    /// Get findings by category
    pub fn get_findings_by_category(&self, category: VulnerabilityCategory) -> Vec<Finding> {
        self.findings
            .read()
            .iter()
            .filter(|f| f.category == category)
            .cloned()
            .collect()
    }

    /// Clear all findings
    pub fn clear_findings(&self) {
        self.findings.write().clear();
        self.vulnerabilities_found.store(0, Ordering::SeqCst);
    }

    /// Get scan statistics
    pub fn get_stats(&self) -> ScanStats {
        let findings = self.findings.read();

        ScanStats {
            is_running: self.is_running.load(Ordering::SeqCst),
            requests_scanned: self.requests_scanned.load(Ordering::SeqCst),
            vulnerabilities_found: self.vulnerabilities_found.load(Ordering::SeqCst),
            critical_count: findings
                .iter()
                .filter(|f| f.severity == Severity::Critical)
                .count(),
            high_count: findings
                .iter()
                .filter(|f| f.severity == Severity::High)
                .count(),
            medium_count: findings
                .iter()
                .filter(|f| f.severity == Severity::Medium)
                .count(),
            low_count: findings
                .iter()
                .filter(|f| f.severity == Severity::Low)
                .count(),
            info_count: findings
                .iter()
                .filter(|f| f.severity == Severity::Info)
                .count(),
        }
    }
}

/// Scan statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanStats {
    pub is_running: bool,
    pub requests_scanned: u64,
    pub vulnerabilities_found: u64,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
}
