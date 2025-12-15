//! Detection Rules - Vulnerability detection patterns and payloads

use super::types::{Finding, ResponseInfo, Severity, VulnerabilityCategory};
use crate::capture::CaptureEntry;
use serde::{Deserialize, Serialize};

/// A detection rule defines how to find a specific vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    /// Unique identifier
    pub id: String,
    /// Human readable name
    pub name: String,
    /// Category
    pub category: VulnerabilityCategory,
    /// Severity if found
    pub severity: Severity,
    /// Description of the vulnerability
    pub description: String,
    /// Patterns to match in request (passive)
    pub request_patterns: Vec<String>,
    /// Patterns to match in response (passive)
    pub response_patterns: Vec<String>,
    /// Header patterns to check
    pub header_patterns: Vec<HeaderPattern>,
    /// Payloads for active testing
    pub active_payloads: Vec<String>,
    /// How to fix this issue
    pub remediation: String,
    /// Reference links
    pub references: Vec<String>,
    /// Whether this rule is enabled
    pub enabled: bool,
}

/// Header pattern for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderPattern {
    pub header_name: String,
    pub pattern: PatternType,
    pub message: String,
}

/// Pattern matching type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PatternType {
    /// Header should exist
    Exists,
    /// Header should NOT exist
    Missing,
    /// Header should contain value
    Contains(String),
    /// Header should NOT contain value
    NotContains(String),
    /// Header should match regex
    Regex(String),
    /// Header should NOT match regex
    NotRegex(String),
}

impl DetectionRule {
    /// Check a captured entry passively
    pub fn check_passive(&self, entry: &CaptureEntry) -> Option<Finding> {
        if !self.enabled {
            return None;
        }

        let request = &entry.request;
        let response = entry.response.as_ref();

        // Check request patterns (in body)
        if !request.body.is_empty() {
            let body_str = String::from_utf8_lossy(&request.body);
            for pattern in &self.request_patterns {
                if body_str.to_lowercase().contains(&pattern.to_lowercase()) {
                    return Some(self.create_finding(
                        &request.url,
                        format!("Suspicious pattern '{}' found in request body", pattern),
                        body_str[..body_str.len().min(200)].to_string(),
                        None,
                        false,
                    ));
                }
            }
        }

        // Check response patterns
        if let Some(resp) = response {
            if !resp.body.is_empty() {
                let body_str = String::from_utf8_lossy(&resp.body);
                for pattern in &self.response_patterns {
                    if body_str.to_lowercase().contains(&pattern.to_lowercase()) {
                        return Some(self.create_finding(
                            &request.url,
                            format!("Suspicious pattern '{}' found in response", pattern),
                            body_str[..body_str.len().min(200)].to_string(),
                            Some(ResponseInfo {
                                status: resp.status_code,
                                length: resp.body.len(),
                                duration_ms: resp.duration_ms as u64,
                            }),
                            false,
                        ));
                    }
                }
            }

            // Check headers
            for header_pattern in &self.header_patterns {
                let header_value = resp
                    .headers
                    .iter()
                    .find(|(k, _)| k.to_lowercase() == header_pattern.header_name.to_lowercase())
                    .map(|(_, v)| v.clone());

                let is_vulnerable = match &header_pattern.pattern {
                    PatternType::Exists => header_value.is_none(),
                    PatternType::Missing => header_value.is_some(),
                    PatternType::Contains(val) => header_value
                        .as_ref()
                        .map(|v| !v.to_lowercase().contains(&val.to_lowercase()))
                        .unwrap_or(true),
                    PatternType::NotContains(val) => header_value
                        .as_ref()
                        .map(|v| v.to_lowercase().contains(&val.to_lowercase()))
                        .unwrap_or(false),
                    PatternType::Regex(pattern) => {
                        if let Ok(re) = regex::Regex::new(pattern) {
                            header_value
                                .as_ref()
                                .map(|v| !re.is_match(v))
                                .unwrap_or(true)
                        } else {
                            false
                        }
                    }
                    PatternType::NotRegex(pattern) => {
                        if let Ok(re) = regex::Regex::new(pattern) {
                            header_value
                                .as_ref()
                                .map(|v| re.is_match(v))
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    }
                };

                if is_vulnerable {
                    return Some(self.create_finding(
                        &request.url,
                        header_pattern.message.clone(),
                        format!(
                            "Header: {} = {:?}",
                            header_pattern.header_name,
                            header_value.unwrap_or_default()
                        ),
                        Some(ResponseInfo {
                            status: resp.status_code,
                            length: resp.body.len(),
                            duration_ms: resp.duration_ms as u64,
                        }),
                        true,
                    ));
                }
            }
        }

        None
    }

    fn create_finding(
        &self,
        url: &str,
        description: String,
        evidence: String,
        response: Option<ResponseInfo>,
        confirmed: bool,
    ) -> Finding {
        Finding {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id: self.id.clone(),
            category: self.category.clone(),
            severity: self.severity.clone(),
            title: self.name.clone(),
            description,
            url: url.to_string(),
            evidence,
            request: None,
            response,
            remediation: self.remediation.clone(),
            references: self.references.clone(),
            timestamp: chrono::Utc::now(),
            confirmed,
        }
    }

    /// Get default security rules
    pub fn default_rules() -> Vec<DetectionRule> {
        vec![
            // SQL Injection
            DetectionRule {
                id: "sqli-error-based".to_string(),
                name: "SQL Injection (Error-Based)".to_string(),
                category: VulnerabilityCategory::Injection,
                severity: Severity::High,
                description: "SQL injection vulnerability detected through error messages"
                    .to_string(),
                request_patterns: vec![],
                response_patterns: vec![
                    "sql syntax".to_string(),
                    "mysql_fetch".to_string(),
                    "ORA-".to_string(),
                    "syntax error".to_string(),
                    "unclosed quotation".to_string(),
                    "pg_query".to_string(),
                    "SQLite3::".to_string(),
                    "microsoft ole db".to_string(),
                    "ODBC SQL Server Driver".to_string(),
                    "PostgreSQL query failed".to_string(),
                    "quoted string not properly terminated".to_string(),
                ],
                header_patterns: vec![],
                active_payloads: vec![
                    "'".to_string(),
                    "\"".to_string(),
                    "' OR '1'='1".to_string(),
                    "\" OR \"1\"=\"1".to_string(),
                    "1' ORDER BY 1--".to_string(),
                    "1 UNION SELECT NULL--".to_string(),
                    "'; DROP TABLE users--".to_string(),
                    "1; WAITFOR DELAY '0:0:5'--".to_string(),
                ],
                remediation: "Use parameterized queries or prepared statements. Never concatenate user input directly into SQL queries.".to_string(),
                references: vec![
                    "https://owasp.org/www-community/attacks/SQL_Injection".to_string(),
                    "https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html".to_string(),
                ],
                enabled: true,
            },
            // XSS
            DetectionRule {
                id: "xss-reflected".to_string(),
                name: "Cross-Site Scripting (Reflected)".to_string(),
                category: VulnerabilityCategory::XSS,
                severity: Severity::Medium,
                description: "Reflected XSS vulnerability - user input reflected in response"
                    .to_string(),
                request_patterns: vec![],
                response_patterns: vec![],
                header_patterns: vec![],
                active_payloads: vec![
                    "<script>alert(1)</script>".to_string(),
                    "<img src=x onerror=alert(1)>".to_string(),
                    "<svg onload=alert(1)>".to_string(),
                    "javascript:alert(1)".to_string(),
                    "<body onload=alert(1)>".to_string(),
                    "'-alert(1)-'".to_string(),
                    "\"><script>alert(1)</script>".to_string(),
                ],
                remediation: "Encode all user input before rendering in HTML. Use Content-Security-Policy headers.".to_string(),
                references: vec![
                    "https://owasp.org/www-community/attacks/xss/".to_string(),
                    "https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html".to_string(),
                ],
                enabled: true,
            },
            // Path Traversal
            DetectionRule {
                id: "path-traversal".to_string(),
                name: "Path Traversal".to_string(),
                category: VulnerabilityCategory::PathTraversal,
                severity: Severity::High,
                description: "Path traversal vulnerability allowing access to files outside web root".to_string(),
                request_patterns: vec![],
                response_patterns: vec![
                    "root:x:0:0".to_string(),
                    "[boot loader]".to_string(),
                    "\\windows\\system32".to_string(),
                    "/etc/passwd".to_string(),
                ],
                header_patterns: vec![],
                active_payloads: vec![
                    "../../../etc/passwd".to_string(),
                    "..\\..\\..\\windows\\win.ini".to_string(),
                    "....//....//....//etc/passwd".to_string(),
                    "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc/passwd".to_string(),
                    "..%252f..%252f..%252fetc/passwd".to_string(),
                ],
                remediation: "Validate and sanitize file paths. Use allowlists for permitted files.".to_string(),
                references: vec![
                    "https://owasp.org/www-community/attacks/Path_Traversal".to_string(),
                ],
                enabled: true,
            },
            // Missing Security Headers
            DetectionRule {
                id: "missing-csp".to_string(),
                name: "Missing Content-Security-Policy".to_string(),
                category: VulnerabilityCategory::SecurityMisconfiguration,
                severity: Severity::Low,
                description: "Content-Security-Policy header is missing".to_string(),
                request_patterns: vec![],
                response_patterns: vec![],
                header_patterns: vec![HeaderPattern {
                    header_name: "Content-Security-Policy".to_string(),
                    pattern: PatternType::Exists,
                    message: "Content-Security-Policy header is not set".to_string(),
                }],
                active_payloads: vec![],
                remediation: "Add Content-Security-Policy header to prevent XSS and data injection attacks.".to_string(),
                references: vec![
                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP".to_string(),
                ],
                enabled: true,
            },
            DetectionRule {
                id: "missing-xfo".to_string(),
                name: "Missing X-Frame-Options".to_string(),
                category: VulnerabilityCategory::SecurityMisconfiguration,
                severity: Severity::Low,
                description: "X-Frame-Options header is missing, allowing clickjacking".to_string(),
                request_patterns: vec![],
                response_patterns: vec![],
                header_patterns: vec![HeaderPattern {
                    header_name: "X-Frame-Options".to_string(),
                    pattern: PatternType::Exists,
                    message: "X-Frame-Options header is not set".to_string(),
                }],
                active_payloads: vec![],
                remediation: "Add X-Frame-Options: DENY or SAMEORIGIN header.".to_string(),
                references: vec![
                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options".to_string(),
                ],
                enabled: true,
            },
            DetectionRule {
                id: "missing-xcto".to_string(),
                name: "Missing X-Content-Type-Options".to_string(),
                category: VulnerabilityCategory::SecurityMisconfiguration,
                severity: Severity::Low,
                description: "X-Content-Type-Options header is missing".to_string(),
                request_patterns: vec![],
                response_patterns: vec![],
                header_patterns: vec![HeaderPattern {
                    header_name: "X-Content-Type-Options".to_string(),
                    pattern: PatternType::Exists,
                    message: "X-Content-Type-Options header is not set".to_string(),
                }],
                active_payloads: vec![],
                remediation: "Add X-Content-Type-Options: nosniff header.".to_string(),
                references: vec![
                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options".to_string(),
                ],
                enabled: true,
            },
            // Information Disclosure
            DetectionRule {
                id: "server-version".to_string(),
                name: "Server Version Disclosure".to_string(),
                category: VulnerabilityCategory::InformationDisclosure,
                severity: Severity::Info,
                description: "Server version information is disclosed in headers".to_string(),
                request_patterns: vec![],
                response_patterns: vec![],
                header_patterns: vec![HeaderPattern {
                    header_name: "Server".to_string(),
                    pattern: PatternType::NotRegex(r"(Apache|nginx|IIS|Express)/[\d\.]+".to_string()),
                    message: "Server version is disclosed in headers".to_string(),
                }],
                active_payloads: vec![],
                remediation: "Remove or obfuscate server version information from response headers.".to_string(),
                references: vec![],
                enabled: true,
            },
            DetectionRule {
                id: "sensitive-data-exposure".to_string(),
                name: "Sensitive Data in Response".to_string(),
                category: VulnerabilityCategory::SensitiveDataExposure,
                severity: Severity::Medium,
                description: "Sensitive data patterns detected in response".to_string(),
                request_patterns: vec![],
                response_patterns: vec![
                    "password".to_string(),
                    "api_key".to_string(),
                    "apikey".to_string(),
                    "secret_key".to_string(),
                    "private_key".to_string(),
                    "-----BEGIN RSA PRIVATE KEY-----".to_string(),
                    "-----BEGIN PRIVATE KEY-----".to_string(),
                    "aws_access_key_id".to_string(),
                    "aws_secret_access_key".to_string(),
                ],
                header_patterns: vec![],
                active_payloads: vec![],
                remediation: "Ensure sensitive data is not exposed in API responses. Use proper access controls.".to_string(),
                references: vec![
                    "https://owasp.org/www-project-top-ten/2017/A3_2017-Sensitive_Data_Exposure".to_string(),
                ],
                enabled: true,
            },
            // Open Redirect
            DetectionRule {
                id: "open-redirect".to_string(),
                name: "Open Redirect".to_string(),
                category: VulnerabilityCategory::OpenRedirect,
                severity: Severity::Medium,
                description: "Application may redirect to untrusted URLs".to_string(),
                request_patterns: vec![
                    "redirect=".to_string(),
                    "url=".to_string(),
                    "next=".to_string(),
                    "return=".to_string(),
                    "returnUrl=".to_string(),
                    "goto=".to_string(),
                ],
                response_patterns: vec![],
                header_patterns: vec![],
                active_payloads: vec![
                    "https://evil.com".to_string(),
                    "//evil.com".to_string(),
                    "/\\evil.com".to_string(),
                    "https:evil.com".to_string(),
                ],
                remediation: "Validate redirect URLs against an allowlist. Avoid using user input in redirects.".to_string(),
                references: vec![
                    "https://cheatsheetseries.owasp.org/cheatsheets/Unvalidated_Redirects_and_Forwards_Cheat_Sheet.html".to_string(),
                ],
                enabled: true,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules_exist() {
        let rules = DetectionRule::default_rules();
        assert!(!rules.is_empty());
        assert!(rules
            .iter()
            .any(|r| r.category == VulnerabilityCategory::Injection));
        assert!(rules
            .iter()
            .any(|r| r.category == VulnerabilityCategory::XSS));
    }
}
