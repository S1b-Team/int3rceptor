//! Detection Rules - Vulnerability detection patterns and payloads

use super::types::{Finding, ResponseInfo, Severity, VulnerabilityCategory};
use crate::capture::CaptureEntry;
use serde::{Deserialize, Serialize};
use serde_json;
use tracing::warn;
#[cfg(feature = "pro")]
const PREMIUM_RULES_JSON: &str = include_str!("../../../core-pro/src/rules/premium_rules.json");

#[cfg(feature = "pro")]
fn load_premium_rules() -> Vec<DetectionRule> {
    serde_json::from_str(PREMIUM_RULES_JSON).unwrap_or_default()
}

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
        let mut rules: Vec<DetectionRule> = serde_json::from_str(include_str!("free_rules.json"))
            .unwrap_or_default();

        #[cfg(feature = "pro")]
        {
            if let Some(valid_rules) = load_premium_rules_with_license() {
                rules.extend(valid_rules);
            } else {
                warn!("Premium rules not loaded: missing or invalid license (INT3RCEPTOR_LICENSE)");
            }
        }

        if rules.is_empty() {
            warn!("Free ruleset is empty in OSS build; premium tier required for scanning.");
        }

        rules
    }

}

#[cfg(feature = "pro")]
fn load_premium_rules_with_license() -> Option<Vec<DetectionRule>> {
    let key = std::env::var("INT3RCEPTOR_LICENSE").ok()?;
    if !core_pro::licensing::validate_license(&key) {
        return None;
    }
    Some(load_premium_rules())
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
