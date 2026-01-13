use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScopeConfig {
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Clone, Default)]
pub struct ScopeManager {
    config: Arc<RwLock<ScopeConfig>>,
}

impl ScopeManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ScopeConfig::default())),
        }
    }

    pub fn get_config(&self) -> ScopeConfig {
        self.config.read().clone()
    }

    pub fn set_config(&self, config: ScopeConfig) {
        *self.config.write() = config;
    }

    pub fn is_in_scope(&self, url: &str) -> bool {
        let config = self.config.read();

        // If excluded, it's out of scope immediately
        for pattern in &config.excludes {
            if url.contains(pattern) {
                return false;
            }
        }

        // If includes is empty, everything is in scope (unless excluded)
        if config.includes.is_empty() {
            return true;
        }

        // Must match at least one include
        for pattern in &config.includes {
            if url.contains(pattern) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_scope_allows_all() {
        let manager = ScopeManager::new();
        assert!(manager.is_in_scope("https://example.com"));
        assert!(manager.is_in_scope("https://any-domain.org/path"));
    }

    #[test]
    fn test_include_patterns() {
        let manager = ScopeManager::new();
        manager.set_config(ScopeConfig {
            includes: vec!["example.com".to_string(), "test.org".to_string()],
            excludes: vec![],
        });

        assert!(manager.is_in_scope("https://example.com/api"));
        assert!(manager.is_in_scope("https://sub.example.com"));
        assert!(manager.is_in_scope("https://test.org"));
        assert!(!manager.is_in_scope("https://other.com"));
    }

    #[test]
    fn test_exclude_patterns() {
        let manager = ScopeManager::new();
        manager.set_config(ScopeConfig {
            includes: vec![],
            excludes: vec!["logout".to_string(), "static".to_string()],
        });

        assert!(manager.is_in_scope("https://example.com/api"));
        assert!(!manager.is_in_scope("https://example.com/logout"));
        assert!(!manager.is_in_scope("https://example.com/static/js/app.js"));
    }

    #[test]
    fn test_exclude_takes_precedence() {
        let manager = ScopeManager::new();
        manager.set_config(ScopeConfig {
            includes: vec!["example.com".to_string()],
            excludes: vec!["example.com/admin".to_string()],
        });

        assert!(manager.is_in_scope("https://example.com/api"));
        assert!(!manager.is_in_scope("https://example.com/admin/users"));
    }

    #[test]
    fn test_get_set_config() {
        let manager = ScopeManager::new();
        let config = ScopeConfig {
            includes: vec!["test.com".to_string()],
            excludes: vec!["blocked".to_string()],
        };
        manager.set_config(config.clone());

        let retrieved = manager.get_config();
        assert_eq!(retrieved.includes, config.includes);
        assert_eq!(retrieved.excludes, config.excludes);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let manager = ScopeManager::new();
        let manager_clone = manager.clone();

        let handle = thread::spawn(move || {
            manager_clone.set_config(ScopeConfig {
                includes: vec!["thread.com".to_string()],
                excludes: vec![],
            });
        });

        handle.join().unwrap();
        assert!(manager.is_in_scope("https://thread.com/test"));
    }
}
