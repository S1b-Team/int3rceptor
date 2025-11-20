use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

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
        self.config.read().unwrap().clone()
    }

    pub fn set_config(&self, config: ScopeConfig) {
        *self.config.write().unwrap() = config;
    }

    pub fn is_in_scope(&self, url: &str) -> bool {
        let config = self.config.read().unwrap();

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
