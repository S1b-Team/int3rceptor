//! VOIDWALKER Bridge
//!
//! Integrates INT3RCEPTOR's WebSocket handling with VOIDWALKER's analysis system.
//!
//! This module handles the capture and routing of WebSocket frames from
//! INT3RCEPTOR to VOIDWALKER for deep inspection and analysis.

use parking_lot::RwLock;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// Configuration for the VOIDWALKER bridge
#[derive(Debug, Clone)]
pub struct VoidwalkerBridgeConfig {
    /// Whether to enable WebSocket capture
    pub enabled: bool,
    /// Maximum frames to buffer per connection
    pub max_frames_per_connection: usize,
    /// Maximum simultaneous connections to track
    pub max_connections: usize,
}

impl Default for VoidwalkerBridgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_frames_per_connection: 1000,
            max_connections: 100,
        }
    }
}

/// Bridge between INT3RCEPTOR WebSocket proxy and VOIDWALKER analysis
pub struct VoidwalkerBridge {
    /// Configuration
    config: VoidwalkerBridgeConfig,
    /// Active connections
    connections: RwLock<HashMap<String, WebSocketConnectionInfo>>,
    /// Event notifier
    event_tx: broadcast::Sender<VoidwalkerEvent>,
    /// Connection counter
    connection_counter: std::sync::atomic::AtomicU64,
}

/// Information about an active WebSocket connection
#[derive(Debug, Clone)]
pub struct WebSocketConnectionInfo {
    /// Connection ID
    pub id: String,
    /// Associated HTTP request ID
    pub http_request_id: u64,
    /// Target URL
    pub url: String,
    /// Host
    pub host: String,
    /// Negotiated subprotocol
    pub protocol: Option<String>,
    /// Is secure (wss)
    pub secure: bool,
    /// Connection state
    pub state: ConnectionState,
    /// Frame count
    pub frame_count: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Open,
    Closing,
    Closed,
}

/// Frame direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameDirection {
    Sent,
    Received,
}

/// Events emitted by the VOIDWALKER bridge
#[derive(Debug, Clone)]
pub enum VoidwalkerEvent {
    /// New WebSocket connection
    ConnectionOpened {
        connection_id: String,
        http_request_id: u64,
        url: String,
        protocol: Option<String>,
    },
    /// WebSocket connection closed
    ConnectionClosed {
        connection_id: String,
        code: Option<u16>,
        reason: Option<String>,
    },
    /// Frame captured
    FrameCaptured {
        connection_id: String,
        direction: String,
        frame_type: String,
        length: usize,
    },
}

impl VoidwalkerBridge {
    /// Create a new VOIDWALKER bridge
    pub fn new(config: VoidwalkerBridgeConfig) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            config,
            connections: RwLock::new(HashMap::new()),
            event_tx: tx,
            connection_counter: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Subscribe to bridge events
    pub fn subscribe(&self) -> broadcast::Receiver<VoidwalkerEvent> {
        self.event_tx.subscribe()
    }

    /// Register a new WebSocket connection
    pub fn register_connection(
        &self,
        http_request_id: u64,
        url: String,
        host: String,
        secure: bool,
    ) -> Option<String> {
        if !self.config.enabled {
            return None;
        }

        let mut conns = self.connections.write();

        // Check capacity
        if conns.len() >= self.config.max_connections {
            // Remove oldest closed connection
            let closed: Vec<_> = conns
                .iter()
                .filter(|(_, c)| c.state == ConnectionState::Closed)
                .map(|(id, _)| id.clone())
                .take(1)
                .collect();

            for id in closed {
                conns.remove(&id);
            }

            if conns.len() >= self.config.max_connections {
                warn!("VOIDWALKER bridge: connection limit reached");
                return None;
            }
        }

        let id = format!(
            "ws-{}",
            self.connection_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        );

        let conn_info = WebSocketConnectionInfo {
            id: id.clone(),
            http_request_id,
            url: url.clone(),
            host,
            protocol: None,
            secure,
            state: ConnectionState::Connecting,
            frame_count: 0,
            bytes_sent: 0,
            bytes_received: 0,
        };

        conns.insert(id.clone(), conn_info);

        let _ = self.event_tx.send(VoidwalkerEvent::ConnectionOpened {
            connection_id: id.clone(),
            http_request_id,
            url,
            protocol: None,
        });

        info!("VOIDWALKER bridge: registered connection {}", id);
        Some(id)
    }

    /// Mark connection as open with negotiated protocol
    pub fn connection_opened(&self, connection_id: &str, protocol: Option<String>) {
        let mut conns = self.connections.write();
        if let Some(conn) = conns.get_mut(connection_id) {
            conn.state = ConnectionState::Open;
            conn.protocol = protocol;
            debug!("VOIDWALKER bridge: connection {} opened", connection_id);
        }
    }

    /// Mark connection as closed
    pub fn connection_closed(
        &self,
        connection_id: &str,
        code: Option<u16>,
        reason: Option<String>,
    ) {
        let mut conns = self.connections.write();
        if let Some(conn) = conns.get_mut(connection_id) {
            conn.state = ConnectionState::Closed;

            let _ = self.event_tx.send(VoidwalkerEvent::ConnectionClosed {
                connection_id: connection_id.to_string(),
                code,
                reason,
            });

            info!("VOIDWALKER bridge: connection {} closed", connection_id);
        }
    }

    /// Record a captured frame
    pub fn record_frame(
        &self,
        connection_id: &str,
        direction: FrameDirection,
        frame_type: &str,
        payload: &[u8],
    ) {
        let mut conns = self.connections.write();
        if let Some(conn) = conns.get_mut(connection_id) {
            conn.frame_count += 1;

            match direction {
                FrameDirection::Sent => conn.bytes_sent += payload.len() as u64,
                FrameDirection::Received => conn.bytes_received += payload.len() as u64,
            }

            let _ = self.event_tx.send(VoidwalkerEvent::FrameCaptured {
                connection_id: connection_id.to_string(),
                direction: match direction {
                    FrameDirection::Sent => "sent".to_string(),
                    FrameDirection::Received => "received".to_string(),
                },
                frame_type: frame_type.to_string(),
                length: payload.len(),
            });

            debug!(
                "VOIDWALKER bridge: frame recorded for {} ({} bytes)",
                connection_id,
                payload.len()
            );
        }
    }

    /// Get connection info
    pub fn get_connection(&self, connection_id: &str) -> Option<WebSocketConnectionInfo> {
        self.connections.read().get(connection_id).cloned()
    }

    /// List all connections
    pub fn list_connections(&self) -> Vec<WebSocketConnectionInfo> {
        self.connections.read().values().cloned().collect()
    }

    /// List active connections
    pub fn active_connections(&self) -> Vec<WebSocketConnectionInfo> {
        self.connections
            .read()
            .values()
            .filter(|c| c.state == ConnectionState::Open)
            .cloned()
            .collect()
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.read().len()
    }

    /// Check if the bridge is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

impl Default for VoidwalkerBridge {
    fn default() -> Self {
        Self::new(VoidwalkerBridgeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_connection() {
        let bridge = VoidwalkerBridge::default();

        let conn_id = bridge.register_connection(
            1,
            "wss://example.com/ws".to_string(),
            "example.com".to_string(),
            true,
        );

        assert!(conn_id.is_some());
        assert!(conn_id.unwrap().starts_with("ws-"));
        assert_eq!(bridge.connection_count(), 1);
    }

    #[test]
    fn test_record_frame() {
        let bridge = VoidwalkerBridge::default();

        let conn_id = bridge
            .register_connection(
                1,
                "wss://example.com/ws".to_string(),
                "example.com".to_string(),
                true,
            )
            .unwrap();

        bridge.connection_opened(&conn_id, Some("graphql-ws".to_string()));
        bridge.record_frame(&conn_id, FrameDirection::Sent, "text", b"hello");
        bridge.record_frame(&conn_id, FrameDirection::Received, "text", b"world");

        let conn = bridge.get_connection(&conn_id).unwrap();
        assert_eq!(conn.frame_count, 2);
        assert_eq!(conn.bytes_sent, 5);
        assert_eq!(conn.bytes_received, 5);
    }
}
