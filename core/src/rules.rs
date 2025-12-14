use parking_lot::RwLock;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    Request,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchCondition {
    // Simple substring matching
    UrlContains(String),
    HeaderContains(String, String), // Key, Value substring
    BodyContains(String),

    // Advanced regex matching
    UrlRegex(String),
    HeaderRegex(String, String), // Key, Value regex
    BodyRegex(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // Simple replacements
    ReplaceBody(String, String), // Target, Replacement
    SetHeader(String, String),   // Key, Value
    RemoveHeader(String),        // Key

    // Advanced regex replacements with capture groups
    RegexReplaceBody(String, String), // Regex pattern, Replacement (supports $1, $2, etc.)
    RegexReplaceHeader(String, String, String), // Header key, Regex pattern, Replacement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub active: bool,
    pub rule_type: RuleType,
    pub condition: MatchCondition,
    pub action: Action,
}

#[derive(Clone)]
pub struct RuleEngine {
    rules: Arc<RwLock<Vec<Rule>>>,
    // Regex cache for performance (avoid recompiling)
    regex_cache: Arc<RwLock<HashMap<String, Regex>>>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            regex_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_rule(&self, rule: Rule) {
        self.rules.write().push(rule);
    }

    pub fn get_rules(&self) -> Vec<Rule> {
        self.rules.read().clone()
    }

    pub fn clear_rules(&self) {
        self.rules.write().clear();
        // Also clear regex cache
        self.regex_cache.write().clear();
    }

    /// Get or compile a regex pattern (with caching)
    fn get_regex(&self, pattern: &str) -> Option<Regex> {
        // Check cache first
        {
            let cache = self.regex_cache.read();
            if let Some(regex) = cache.get(pattern) {
                return Some(regex.clone());
            }
        }

        // Compile and cache
        match Regex::new(pattern) {
            Ok(regex) => {
                self.regex_cache
                    .write()
                    .insert(pattern.to_string(), regex.clone());
                Some(regex)
            }
            Err(e) => {
                tracing::warn!("Invalid regex pattern '{}': {}", pattern, e);
                None
            }
        }
    }

    pub fn apply_request_rules(&self, parts: &mut http::request::Parts, body: &mut Vec<u8>) {
        let rules = self.rules.read();
        for rule in rules.iter() {
            if !rule.active || rule.rule_type != RuleType::Request {
                continue;
            }

            if self.matches_request(rule, parts, body) {
                self.execute_action(rule, &mut parts.headers, body);
            }
        }
    }

    pub fn apply_response_rules(&self, parts: &mut http::response::Parts, body: &mut Vec<u8>) {
        let rules = self.rules.read();
        for rule in rules.iter() {
            if !rule.active || rule.rule_type != RuleType::Response {
                continue;
            }

            if self.matches_response(rule, parts, body) {
                self.execute_action(rule, &mut parts.headers, body);
            }
        }
    }

    fn matches_request(&self, rule: &Rule, parts: &http::request::Parts, body: &[u8]) -> bool {
        match &rule.condition {
            MatchCondition::UrlContains(s) => parts.uri.to_string().contains(s),
            MatchCondition::UrlRegex(pattern) => {
                if let Some(regex) = self.get_regex(pattern) {
                    regex.is_match(&parts.uri.to_string())
                } else {
                    false
                }
            }
            MatchCondition::HeaderContains(k, v) => parts
                .headers
                .get(k)
                .and_then(|val| val.to_str().ok())
                .map(|val| val.contains(v))
                .unwrap_or(false),
            MatchCondition::HeaderRegex(k, pattern) => {
                if let Some(regex) = self.get_regex(pattern) {
                    parts
                        .headers
                        .get(k)
                        .and_then(|val| val.to_str().ok())
                        .map(|val| regex.is_match(val))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            MatchCondition::BodyContains(s) => String::from_utf8_lossy(body).contains(s),
            MatchCondition::BodyRegex(pattern) => {
                if let Some(regex) = self.get_regex(pattern) {
                    regex.is_match(&String::from_utf8_lossy(body))
                } else {
                    false
                }
            }
        }
    }

    fn matches_response(&self, rule: &Rule, _parts: &http::response::Parts, body: &[u8]) -> bool {
        match &rule.condition {
            MatchCondition::UrlContains(_) => false, // Response doesn't have URI
            MatchCondition::UrlRegex(_) => false,
            MatchCondition::HeaderContains(k, v) => _parts
                .headers
                .get(k)
                .and_then(|val| val.to_str().ok())
                .map(|val| val.contains(v))
                .unwrap_or(false),
            MatchCondition::HeaderRegex(k, pattern) => {
                if let Some(regex) = self.get_regex(pattern) {
                    _parts
                        .headers
                        .get(k)
                        .and_then(|val| val.to_str().ok())
                        .map(|val| regex.is_match(val))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            MatchCondition::BodyContains(s) => String::from_utf8_lossy(body).contains(s),
            MatchCondition::BodyRegex(pattern) => {
                if let Some(regex) = self.get_regex(pattern) {
                    regex.is_match(&String::from_utf8_lossy(body))
                } else {
                    false
                }
            }
        }
    }

    fn execute_action(&self, rule: &Rule, headers: &mut http::HeaderMap, body: &mut Vec<u8>) {
        match &rule.action {
            Action::ReplaceBody(target, replacement) => {
                let s = String::from_utf8_lossy(body).to_string();
                let new_s = s.replace(target, replacement);
                *body = new_s.into_bytes();

                // Update Content-Length if present
                if headers.contains_key(http::header::CONTENT_LENGTH) {
                    if let Ok(val) = http::HeaderValue::from_str(&body.len().to_string()) {
                        headers.insert(http::header::CONTENT_LENGTH, val);
                    }
                }
            }
            Action::RegexReplaceBody(pattern, replacement) => {
                if let Some(regex) = self.get_regex(pattern) {
                    let s = String::from_utf8_lossy(body).to_string();
                    let new_s = regex.replace_all(&s, replacement.as_str()).to_string();
                    *body = new_s.into_bytes();

                    // Update Content-Length if present
                    if headers.contains_key(http::header::CONTENT_LENGTH) {
                        if let Ok(val) = http::HeaderValue::from_str(&body.len().to_string()) {
                            headers.insert(http::header::CONTENT_LENGTH, val);
                        }
                    }
                }
            }
            Action::SetHeader(k, v) => {
                if let (Ok(k), Ok(v)) = (
                    http::header::HeaderName::from_bytes(k.as_bytes()),
                    http::header::HeaderValue::from_str(v),
                ) {
                    headers.insert(k, v);
                }
            }
            Action::RegexReplaceHeader(header_key, pattern, replacement) => {
                if let Some(regex) = self.get_regex(pattern) {
                    if let Ok(k) = http::header::HeaderName::from_bytes(header_key.as_bytes()) {
                        if let Some(val) = headers.get(&k) {
                            if let Ok(val_str) = val.to_str() {
                                let new_val =
                                    regex.replace_all(val_str, replacement.as_str()).to_string();
                                if let Ok(new_header_val) =
                                    http::header::HeaderValue::from_str(&new_val)
                                {
                                    headers.insert(k, new_header_val);
                                }
                            }
                        }
                    }
                }
            }
            Action::RemoveHeader(k) => {
                if let Ok(k) = http::header::HeaderName::from_bytes(k.as_bytes()) {
                    headers.remove(k);
                }
            }
        }
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rule(id: &str, condition: MatchCondition, action: Action) -> Rule {
        Rule {
            id: id.to_string(),
            active: true,
            rule_type: RuleType::Request,
            condition,
            action,
        }
    }

    #[test]
    fn test_add_and_get_rules() {
        let engine = RuleEngine::new();
        let rule = create_test_rule(
            "test1",
            MatchCondition::UrlContains("/api".to_string()),
            Action::SetHeader("X-Test".to_string(), "value".to_string()),
        );

        engine.add_rule(rule);
        let rules = engine.get_rules();

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "test1");
    }

    #[test]
    fn test_clear_rules() {
        let engine = RuleEngine::new();
        engine.add_rule(create_test_rule(
            "r1",
            MatchCondition::UrlContains("/test".to_string()),
            Action::RemoveHeader("X-Remove".to_string()),
        ));
        engine.add_rule(create_test_rule(
            "r2",
            MatchCondition::BodyContains("secret".to_string()),
            Action::ReplaceBody("secret".to_string(), "***".to_string()),
        ));

        assert_eq!(engine.get_rules().len(), 2);
        engine.clear_rules();
        assert_eq!(engine.get_rules().len(), 0);
    }

    #[test]
    fn test_regex_caching() {
        let engine = RuleEngine::new();
        let pattern = r"\d{3}-\d{4}";

        // First call compiles and caches
        let regex1 = engine.get_regex(pattern);
        assert!(regex1.is_some());

        // Second call should use cache
        let regex2 = engine.get_regex(pattern);
        assert!(regex2.is_some());

        // Both should work the same
        assert!(regex1.unwrap().is_match("123-4567"));
    }

    #[test]
    fn test_invalid_regex_returns_none() {
        let engine = RuleEngine::new();
        let invalid_pattern = r"[invalid(";

        let result = engine.get_regex(invalid_pattern);
        assert!(result.is_none());
    }

    #[test]
    fn test_body_replace_action() {
        let engine = RuleEngine::new();
        engine.add_rule(create_test_rule(
            "replace",
            MatchCondition::BodyContains("password".to_string()),
            Action::ReplaceBody("password".to_string(), "********".to_string()),
        ));

        let mut body = b"my password is secret".to_vec();
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::CONTENT_LENGTH,
            http::HeaderValue::from_static("21"),
        );

        // Create minimal request parts
        let (mut parts, _) = http::Request::builder()
            .uri("/api/login")
            .body(())
            .unwrap()
            .into_parts();

        engine.apply_request_rules(&mut parts, &mut body);

        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("********"));
        assert!(!body_str.contains("password"));
    }

    #[test]
    fn test_url_contains_condition() {
        let engine = RuleEngine::new();
        engine.add_rule(create_test_rule(
            "api-rule",
            MatchCondition::UrlContains("/api/v1".to_string()),
            Action::SetHeader("X-API-Version".to_string(), "1".to_string()),
        ));

        let mut body = vec![];

        // Matching URL
        let (mut parts, _) = http::Request::builder()
            .uri("/api/v1/users")
            .body(())
            .unwrap()
            .into_parts();

        engine.apply_request_rules(&mut parts, &mut body);
        assert!(parts.headers.contains_key("x-api-version"));

        // Non-matching URL
        let (mut parts2, _) = http::Request::builder()
            .uri("/web/home")
            .body(())
            .unwrap()
            .into_parts();

        engine.apply_request_rules(&mut parts2, &mut body);
        assert!(!parts2.headers.contains_key("x-api-version"));
    }

    #[test]
    fn test_url_regex_condition() {
        let engine = RuleEngine::new();
        engine.add_rule(create_test_rule(
            "user-id-rule",
            MatchCondition::UrlRegex(r"/users/\d+".to_string()),
            Action::SetHeader("X-Has-User-Id".to_string(), "true".to_string()),
        ));

        let mut body = vec![];

        // Matching URL with numeric ID
        let (mut parts, _) = http::Request::builder()
            .uri("/users/12345")
            .body(())
            .unwrap()
            .into_parts();

        engine.apply_request_rules(&mut parts, &mut body);
        assert!(parts.headers.contains_key("x-has-user-id"));

        // Non-matching URL (no numeric ID)
        let (mut parts2, _) = http::Request::builder()
            .uri("/users/me")
            .body(())
            .unwrap()
            .into_parts();

        engine.apply_request_rules(&mut parts2, &mut body);
        assert!(!parts2.headers.contains_key("x-has-user-id"));
    }

    #[test]
    fn test_inactive_rule_not_applied() {
        let engine = RuleEngine::new();
        let mut rule = create_test_rule(
            "inactive",
            MatchCondition::UrlContains("/test".to_string()),
            Action::SetHeader("X-Should-Not-Exist".to_string(), "value".to_string()),
        );
        rule.active = false;
        engine.add_rule(rule);

        let mut body = vec![];
        let (mut parts, _) = http::Request::builder()
            .uri("/test/path")
            .body(())
            .unwrap()
            .into_parts();

        engine.apply_request_rules(&mut parts, &mut body);
        assert!(!parts.headers.contains_key("x-should-not-exist"));
    }

    #[test]
    fn test_regex_replace_body() {
        let engine = RuleEngine::new();
        engine.add_rule(create_test_rule(
            "redact-ssn",
            MatchCondition::BodyRegex(r"\d{3}-\d{2}-\d{4}".to_string()),
            Action::RegexReplaceBody(r"\d{3}-\d{2}-\d{4}".to_string(), "XXX-XX-XXXX".to_string()),
        ));

        let mut body = b"SSN: 123-45-6789".to_vec();
        let (mut parts, _) = http::Request::builder()
            .uri("/submit")
            .body(())
            .unwrap()
            .into_parts();

        engine.apply_request_rules(&mut parts, &mut body);

        let body_str = String::from_utf8_lossy(&body);
        assert_eq!(body_str, "SSN: XXX-XX-XXXX");
    }

    #[test]
    fn test_remove_header_action() {
        let engine = RuleEngine::new();
        engine.add_rule(create_test_rule(
            "remove-auth",
            MatchCondition::UrlContains("/public".to_string()),
            Action::RemoveHeader("authorization".to_string()),
        ));

        let mut body = vec![];
        let (mut parts, _) = http::Request::builder()
            .uri("/public/resource")
            .header("authorization", "Bearer secret-token")
            .body(())
            .unwrap()
            .into_parts();

        assert!(parts.headers.contains_key("authorization"));
        engine.apply_request_rules(&mut parts, &mut body);
        assert!(!parts.headers.contains_key("authorization"));
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let engine = RuleEngine::new();
        let engine_clone = engine.clone();

        let handle = thread::spawn(move || {
            engine_clone.add_rule(create_test_rule(
                "thread-rule",
                MatchCondition::UrlContains("/thread".to_string()),
                Action::SetHeader("X-Thread".to_string(), "1".to_string()),
            ));
        });

        handle.join().unwrap();
        assert_eq!(engine.get_rules().len(), 1);
    }
}
