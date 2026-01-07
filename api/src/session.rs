// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                   SESSION MANAGEMENT MODULE                              ║
// ║                   Copyright (c) 2025 S1BGr0uP                             ║
// ║                        All Rights Reserved                                ║
// ║  P3 Security: Session-based CSRF protection (not IP-based spoofing)      ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

use interceptor_core::security::constant_time_compare;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const SESSION_EXPIRY_SECS: u64 = 3600; // 1 hour
const SESSION_COOKIE_NAME: &str = "int3rceptor_session";
const SESSION_COOKIE_VALUE_PREFIX: &str = "session_";

/// Session entry with creation timestamp
#[derive(Debug, Clone)]
pub struct SessionEntry {
    pub session_id: String,
    pub created_at: SystemTime,
}

/// Session manager - generates and validates sessions
pub struct SessionManager {
    sessions: RwLock<HashMap<String, SessionEntry>>,
}

impl SessionManager {
    /// Create new session manager
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Generate new session ID (unique, cryptographically random)
    pub fn generate_session(&self) -> String {
        let session_id = format!("{}{}", SESSION_COOKIE_VALUE_PREFIX, Uuid::new_v4());

        if let Ok(mut sessions) = self.sessions.write() {
            sessions.insert(
                session_id.clone(),
                SessionEntry {
                    session_id: session_id.clone(),
                    created_at: SystemTime::now(),
                },
            );
        }

        session_id
    }

    /// Validate session is active and not expired
    pub fn validate_session(&self, session_id: &str) -> bool {
        if !session_id.starts_with(SESSION_COOKIE_VALUE_PREFIX) {
            return false;
        }

        let sessions = match self.sessions.read() {
            Ok(s) => s,
            Err(_) => return false,
        };

        if let Some(entry) = sessions.get(session_id) {
            // Check if session is expired
            if let Ok(elapsed) = entry.created_at.elapsed() {
                if elapsed > Duration::from_secs(SESSION_EXPIRY_SECS) {
                    return false;
                }
            }
            return true;
        }

        false
    }

    /// Invalidate session (logout)
    pub fn invalidate_session(&self, session_id: &str) {
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.remove(session_id);
        }
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&self) {
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.retain(|_, entry| {
                entry
                    .created_at
                    .elapsed()
                    .map(|elapsed| elapsed < Duration::from_secs(SESSION_EXPIRY_SECS))
                    .unwrap_or(false)
            });
        }
    }

    /// Get session count (for monitoring)
    pub fn session_count(&self) -> usize {
        self.sessions.read().map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_generation() {
        let manager = SessionManager::new();
        let session_id = manager.generate_session();

        assert!(session_id.starts_with(SESSION_COOKIE_VALUE_PREFIX));
        assert!(manager.validate_session(&session_id));
    }

    #[test]
    fn test_session_uniqueness() {
        let manager = SessionManager::new();
        let s1 = manager.generate_session();
        let s2 = manager.generate_session();

        assert_ne!(s1, s2);
        assert!(manager.validate_session(&s1));
        assert!(manager.validate_session(&s2));
    }

    #[test]
    fn test_session_invalidation() {
        let manager = SessionManager::new();
        let session_id = manager.generate_session();

        assert!(manager.validate_session(&session_id));
        manager.invalidate_session(&session_id);
        assert!(!manager.validate_session(&session_id));
    }

    #[test]
    fn test_invalid_session_format() {
        let manager = SessionManager::new();
        assert!(!manager.validate_session("invalid_session"));
        assert!(!manager.validate_session(""));
        assert!(!manager.validate_session("random-uuid"));
    }
}
