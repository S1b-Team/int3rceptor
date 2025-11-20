use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use regex::Regex;
use std::collections::HashMap;

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
        let mut rules = self.rules.write().unwrap();
        rules.push(rule);
    }

    pub fn get_rules(&self) -> Vec<Rule> {
        self.rules.read().unwrap().clone()
    }

    pub fn clear_rules(&self) {
        let mut rules = self.rules.write().unwrap();
        rules.clear();
        // Also clear regex cache
        let mut cache = self.regex_cache.write().unwrap();
        cache.clear();
    }
    
    /// Get or compile a regex pattern (with caching)
    fn get_regex(&self, pattern: &str) -> Option<Regex> {
        // Check cache first
        {
            let cache = self.regex_cache.read().unwrap();
            if let Some(regex) = cache.get(pattern) {
                return Some(regex.clone());
            }
        }
        
        // Compile and cache
        match Regex::new(pattern) {
            Ok(regex) => {
                let mut cache = self.regex_cache.write().unwrap();
                cache.insert(pattern.to_string(), regex.clone());
                Some(regex)
            }
            Err(e) => {
                tracing::warn!("Invalid regex pattern '{}': {}", pattern, e);
                None
            }
        }
    }

    pub fn apply_request_rules(
        &self,
        parts: &mut http::request::Parts,
        body: &mut Vec<u8>,
    ) {
        let rules = self.rules.read().unwrap();
        for rule in rules.iter() {
            if !rule.active || rule.rule_type != RuleType::Request {
                continue;
            }

            if self.matches_request(rule, parts, body) {
                self.execute_action(rule, &mut parts.headers, body);
            }
        }
    }

    pub fn apply_response_rules(
        &self,
        parts: &mut http::response::Parts,
        body: &mut Vec<u8>,
    ) {
        let rules = self.rules.read().unwrap();
        for rule in rules.iter() {
            if !rule.active || rule.rule_type != RuleType::Response {
                continue;
            }

            if self.matches_response(rule, parts, body) {
                self.execute_action(rule, &mut parts.headers, body);
            }
        }
    }

    fn matches_request(
        &self,
        rule: &Rule,
        parts: &http::request::Parts,
        body: &[u8],
    ) -> bool {
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
            MatchCondition::BodyContains(s) => {
                String::from_utf8_lossy(body).contains(s)
            }
            MatchCondition::BodyRegex(pattern) => {
                if let Some(regex) = self.get_regex(pattern) {
                    regex.is_match(&String::from_utf8_lossy(body))
                } else {
                    false
                }
            }
        }
    }

    fn matches_response(
        &self,
        rule: &Rule,
        _parts: &http::response::Parts,
        body: &[u8],
    ) -> bool {
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
            MatchCondition::BodyContains(s) => {
                String::from_utf8_lossy(body).contains(s)
            }
            MatchCondition::BodyRegex(pattern) => {
                if let Some(regex) = self.get_regex(pattern) {
                    regex.is_match(&String::from_utf8_lossy(body))
                } else {
                    false
                }
            }
        }
    }

    fn execute_action(
        &self,
        rule: &Rule,
        headers: &mut http::HeaderMap,
        body: &mut Vec<u8>,
    ) {
        match &rule.action {
            Action::ReplaceBody(target, replacement) => {
                let s = String::from_utf8_lossy(body).to_string();
                let new_s = s.replace(target, replacement);
                *body = new_s.into_bytes();
                
                // Update Content-Length if present
                if headers.contains_key(http::header::CONTENT_LENGTH) {
                     headers.insert(
                        http::header::CONTENT_LENGTH,
                        body.len().to_string().parse().unwrap(),
                    );
                }
            }
            Action::RegexReplaceBody(pattern, replacement) => {
                if let Some(regex) = self.get_regex(pattern) {
                    let s = String::from_utf8_lossy(body).to_string();
                    let new_s = regex.replace_all(&s, replacement.as_str()).to_string();
                    *body = new_s.into_bytes();
                    
                    // Update Content-Length if present
                    if headers.contains_key(http::header::CONTENT_LENGTH) {
                        headers.insert(
                            http::header::CONTENT_LENGTH,
                            body.len().to_string().parse().unwrap(),
                        );
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
                                let new_val = regex.replace_all(val_str, replacement.as_str()).to_string();
                                if let Ok(new_header_val) = http::header::HeaderValue::from_str(&new_val) {
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
