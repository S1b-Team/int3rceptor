//! NOWARU Bridge
//!
//! Converts INT3RCEPTOR capture data to NOWARU's rich HTTP transaction format.
//!
//! This module provides the integration layer between INT3RCEPTOR's raw capture
//! mechanism and NOWARU's intelligent traffic analysis system.

use crate::capture::{CaptureEntry, RequestCapture};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info};

/// Configuration for the NOWARU bridge
#[derive(Debug, Clone)]
pub struct NowaruBridgeConfig {
    /// Whether to enable automatic processing
    pub enabled: bool,
    /// Whether to process requests as they arrive
    pub realtime: bool,
    /// Maximum concurrent processing tasks
    pub max_concurrent: usize,
}

impl Default for NowaruBridgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            realtime: true,
            max_concurrent: 4,
        }
    }
}

/// Bridge between INT3RCEPTOR capture and NOWARU analysis
pub struct NowaruBridge {
    /// Configuration
    config: NowaruBridgeConfig,
    /// Event notifier for processed transactions
    event_tx: broadcast::Sender<NowaruEvent>,
}

/// Events emitted by the NOWARU bridge
#[derive(Debug, Clone)]
pub enum NowaruEvent {
    /// Transaction processed successfully
    TransactionProcessed {
        id: u64,
        method: String,
        url: String,
        status: Option<u16>,
        duration_ms: Option<u128>,
    },
    /// Processing error
    ProcessingError {
        id: u64,
        error: String,
    },
    /// Stats updated
    StatsUpdated {
        total_processed: u64,
        avg_response_time_ms: f64,
    },
}

impl NowaruBridge {
    /// Create a new NOWARU bridge
    pub fn new(config: NowaruBridgeConfig) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            config,
            event_tx: tx,
        }
    }

    /// Subscribe to bridge events
    pub fn subscribe(&self) -> broadcast::Receiver<NowaruEvent> {
        self.event_tx.subscribe()
    }

    /// Process a capture entry
    ///
    /// This method transforms INT3RCEPTOR's CaptureEntry into the format
    /// expected by NOWARU's TrafficAnalyzer.
    pub fn process_entry(&self, entry: &CaptureEntry) -> ProcessedEntry {
        let request = &entry.request;
        let response = entry.response.as_ref();

        ProcessedEntry {
            id: request.id,
            timestamp_ms: request.timestamp_ms,
            method: request.method.clone(),
            url: request.url.clone(),
            request_headers: request.headers.clone(),
            request_body: request.body.clone(),
            tls: request.tls,
            response_status: response.map(|r| r.status_code),
            response_headers: response.map(|r| r.headers.clone()),
            response_body: response.map(|r| r.body.clone()),
            duration_ms: response.map(|r| r.duration_ms),
        }
    }

    /// Start real-time processing of captures
    pub async fn start_realtime_processing(
        &self,
        capture: Arc<RequestCapture>,
    ) {
        if !self.config.enabled || !self.config.realtime {
            info!("NOWARU bridge: real-time processing disabled");
            return;
        }

        let mut rx = capture.subscribe();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(entry) => {
                        let id = entry.request.id;
                        let method = entry.request.method.clone();
                        let url = entry.request.url.clone();
                        let status = entry.response.as_ref().map(|r| r.status_code);
                        let duration_ms = entry.response.as_ref().map(|r| r.duration_ms);

                        let _ = event_tx.send(NowaruEvent::TransactionProcessed {
                            id,
                            method,
                            url,
                            status,
                            duration_ms,
                        });

                        debug!("NOWARU bridge: processed entry {}", id);
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!("NOWARU bridge: lagged {} messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("NOWARU bridge: capture channel closed");
                        break;
                    }
                }
            }
        });

        info!("NOWARU bridge: started real-time processing");
    }

    /// Emit an event
    pub fn emit(&self, event: NowaruEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Check if the bridge is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

impl Default for NowaruBridge {
    fn default() -> Self {
        Self::new(NowaruBridgeConfig::default())
    }
}

/// Intermediate format for processed captures
///
/// This struct provides all the data needed for NOWARU's TrafficAnalyzer
/// without directly depending on the nowaru-lib crate.
#[derive(Debug, Clone)]
pub struct ProcessedEntry {
    pub id: u64,
    pub timestamp_ms: i128,
    pub method: String,
    pub url: String,
    pub request_headers: Vec<(String, String)>,
    pub request_body: Vec<u8>,
    pub tls: bool,
    pub response_status: Option<u16>,
    pub response_headers: Option<Vec<(String, String)>>,
    pub response_body: Option<Vec<u8>>,
    pub duration_ms: Option<u128>,
}

impl ProcessedEntry {
    /// Create from a capture entry
    pub fn from_capture(entry: &CaptureEntry) -> Self {
        let request = &entry.request;
        let response = entry.response.as_ref();

        Self {
            id: request.id,
            timestamp_ms: request.timestamp_ms,
            method: request.method.clone(),
            url: request.url.clone(),
            request_headers: request.headers.clone(),
            request_body: request.body.clone(),
            tls: request.tls,
            response_status: response.map(|r| r.status_code),
            response_headers: response.map(|r| r.headers.clone()),
            response_body: response.map(|r| r.body.clone()),
            duration_ms: response.map(|r| r.duration_ms),
        }
    }

    /// Get host from URL
    pub fn host(&self) -> String {
        if let Some(start) = self.url.find("://") {
            let after_scheme = &self.url[start + 3..];
            after_scheme
                .split('/')
                .next()
                .unwrap_or("")
                .split(':')
                .next()
                .unwrap_or("")
                .to_string()
        } else {
            String::new()
        }
    }

    /// Get path from URL
    pub fn path(&self) -> String {
        if let Some(start) = self.url.find("://") {
            let after_scheme = &self.url[start + 3..];
            if let Some(slash) = after_scheme.find('/') {
                after_scheme[slash..]
                    .split('?')
                    .next()
                    .unwrap_or("/")
                    .to_string()
            } else {
                "/".to_string()
            }
        } else if self.url.starts_with('/') {
            self.url.split('?').next().unwrap_or("/").to_string()
        } else {
            "/".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processed_entry_host() {
        let entry = ProcessedEntry {
            id: 1,
            timestamp_ms: 0,
            method: "GET".to_string(),
            url: "https://example.com:8443/api/test".to_string(),
            request_headers: vec![],
            request_body: vec![],
            tls: true,
            response_status: None,
            response_headers: None,
            response_body: None,
            duration_ms: None,
        };

        assert_eq!(entry.host(), "example.com");
        assert_eq!(entry.path(), "/api/test");
    }
}
