//! Scanner Types - Data structures for vulnerability scanning

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Vulnerability severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn score(&self) -> u8 {
        match self {
            Severity::Critical => 10,
            Severity::High => 8,
            Severity::Medium => 5,
            Severity::Low => 3,
            Severity::Info => 1,
        }
    }
}

/// Vulnerability categories (based on OWASP)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VulnerabilityCategory {
    /// SQL Injection, Command Injection, LDAP Injection, etc.
    Injection,
    /// Cross-Site Scripting
    XSS,
    /// Broken Authentication
    BrokenAuth,
    /// Sensitive Data Exposure
    SensitiveDataExposure,
    /// XML External Entities
    XXE,
    /// Broken Access Control
    BrokenAccessControl,
    /// Security Misconfiguration
    SecurityMisconfiguration,
    /// Cross-Site Request Forgery
    CSRF,
    /// Using Components with Known Vulnerabilities
    VulnerableComponents,
    /// Insufficient Logging
    InsufficientLogging,
    /// Server-Side Request Forgery
    SSRF,
    /// Path/Directory Traversal
    PathTraversal,
    /// Information Disclosure
    InformationDisclosure,
    /// Open Redirect
    OpenRedirect,
    /// Other/Custom
    Other,
}

impl VulnerabilityCategory {
    pub fn display_name(&self) -> &str {
        match self {
            VulnerabilityCategory::Injection => "Injection",
            VulnerabilityCategory::XSS => "Cross-Site Scripting (XSS)",
            VulnerabilityCategory::BrokenAuth => "Broken Authentication",
            VulnerabilityCategory::SensitiveDataExposure => "Sensitive Data Exposure",
            VulnerabilityCategory::XXE => "XML External Entities (XXE)",
            VulnerabilityCategory::BrokenAccessControl => "Broken Access Control",
            VulnerabilityCategory::SecurityMisconfiguration => "Security Misconfiguration",
            VulnerabilityCategory::CSRF => "Cross-Site Request Forgery (CSRF)",
            VulnerabilityCategory::VulnerableComponents => "Vulnerable Components",
            VulnerabilityCategory::InsufficientLogging => "Insufficient Logging",
            VulnerabilityCategory::SSRF => "Server-Side Request Forgery (SSRF)",
            VulnerabilityCategory::PathTraversal => "Path Traversal",
            VulnerabilityCategory::InformationDisclosure => "Information Disclosure",
            VulnerabilityCategory::OpenRedirect => "Open Redirect",
            VulnerabilityCategory::Other => "Other",
        }
    }

    pub fn owasp_id(&self) -> Option<&str> {
        match self {
            VulnerabilityCategory::Injection => Some("A03:2021"),
            VulnerabilityCategory::XSS => Some("A03:2021"),
            VulnerabilityCategory::BrokenAuth => Some("A07:2021"),
            VulnerabilityCategory::SensitiveDataExposure => Some("A02:2021"),
            VulnerabilityCategory::XXE => Some("A05:2021"),
            VulnerabilityCategory::BrokenAccessControl => Some("A01:2021"),
            VulnerabilityCategory::SecurityMisconfiguration => Some("A05:2021"),
            VulnerabilityCategory::CSRF => Some("A01:2021"),
            VulnerabilityCategory::VulnerableComponents => Some("A06:2021"),
            VulnerabilityCategory::InsufficientLogging => Some("A09:2021"),
            VulnerabilityCategory::SSRF => Some("A10:2021"),
            _ => None,
        }
    }
}

/// A security finding/vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique ID
    pub id: String,
    /// Rule that triggered this finding
    pub rule_id: String,
    /// Category
    pub category: VulnerabilityCategory,
    /// Severity level
    pub severity: Severity,
    /// Short title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Affected URL
    pub url: String,
    /// Evidence (snippet of request/response showing vulnerability)
    pub evidence: String,
    /// Original request info
    pub request: Option<RequestInfo>,
    /// Response info
    pub response: Option<ResponseInfo>,
    /// Remediation advice
    pub remediation: String,
    /// Reference links
    pub references: Vec<String>,
    /// When the finding was discovered
    pub timestamp: DateTime<Utc>,
    /// Whether this has been confirmed (vs suspected)
    pub confirmed: bool,
}

/// Request information for a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// Response information for a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseInfo {
    pub status: u16,
    pub length: usize,
    pub duration_ms: u64,
}

/// Scan target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTarget {
    pub url: String,
    pub method: Option<String>,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<String>,
    pub depth: u32,
}

/// Scan result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub scan_id: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub targets_scanned: usize,
    pub requests_sent: u64,
    pub findings: Vec<Finding>,
    pub stats: FindingStats,
}

/// Finding statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FindingStats {
    pub total: usize,
    pub by_severity: SeverityStats,
    pub by_category: std::collections::HashMap<String, usize>,
}

/// Severity breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeverityStats {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
}
