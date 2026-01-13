use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::OffsetDateTime;

// P4 Security: Memory bounds configuration
pub const DEFAULT_MAX_FRAMES_PER_SESSION: usize = 1_000; // Per WebSocket connection
pub const DEFAULT_MAX_TOTAL_FRAMES: usize = 10_000; // Total across all connections
pub const MAX_PAYLOAD_SIZE: usize = 10 * 1024 * 1024; // 10 MB per frame

/// WebSocket frame types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WsFrameType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

/// WebSocket message direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WsDirection {
    ClientToServer,
    ServerToClient,
}

/// Captured WebSocket frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsFrame {
    pub id: i64,
    pub connection_id: String,
    pub timestamp: i64,
    pub direction: WsDirection,
    pub frame_type: WsFrameType,
    pub payload: Vec<u8>,
    pub masked: bool,
}

/// WebSocket connection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsConnection {
    pub id: String,
    pub url: String,
    pub established_at: i64,
    pub closed_at: Option<i64>,
    pub frames_count: usize,
}

/// WebSocket capture manager
/// P4 Security: Memory bounds with per-connection and global limits
#[derive(Clone)]
pub struct WsCapture {
    connections: Arc<RwLock<Vec<WsConnection>>>,
    frames: Arc<RwLock<Vec<WsFrame>>>,
    max_frames: usize,
    max_frames_per_connection: usize,
}

impl WsCapture {
    pub fn new(max_frames: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            frames: Arc::new(RwLock::new(Vec::new())),
            max_frames,
            max_frames_per_connection: DEFAULT_MAX_FRAMES_PER_SESSION,
        }
    }

    /// Create with custom per-connection limits
    pub fn with_limits(max_frames: usize, max_per_connection: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            frames: Arc::new(RwLock::new(Vec::new())),
            max_frames,
            max_frames_per_connection: max_per_connection,
        }
    }

    /// Register a new WebSocket connection
    pub fn register_connection(&self, id: String, url: String) {
        let connection = WsConnection {
            id,
            url,
            established_at: OffsetDateTime::now_utc().unix_timestamp(),
            closed_at: None,
            frames_count: 0,
        };

        let mut connections = self.connections.write();
        connections.push(connection);
    }

    /// Mark a connection as closed
    pub fn close_connection(&self, id: &str) {
        let mut connections = self.connections.write();
        if let Some(conn) = connections.iter_mut().find(|c| c.id == id) {
            conn.closed_at = Some(OffsetDateTime::now_utc().unix_timestamp());
        }
    }

    /// Capture a WebSocket frame with memory bounds checking
    /// P4 Security: Enforces per-connection and global limits
    pub fn capture_frame(
        &self,
        connection_id: String,
        direction: WsDirection,
        frame_type: WsFrameType,
        payload: Vec<u8>,
        masked: bool,
    ) -> Result<i64, String> {
        // P4: Check payload size limit (prevent memory exhaustion)
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(format!(
                "Payload size {} exceeds maximum {} bytes",
                payload.len(),
                MAX_PAYLOAD_SIZE
            ));
        }

        // P4: Check per-connection frame limit
        let connection_frames = {
            let frames = self.frames.read();
            frames.iter().filter(|f| f.connection_id == connection_id).count()
        };

        if connection_frames >= self.max_frames_per_connection {
            tracing::warn!(
                connection_id = %connection_id,
                frame_count = connection_frames,
                max_per_connection = self.max_frames_per_connection,
                "WebSocket connection frame limit reached - dropping old frames"
            );
        }

        let frame_id = {
            let frames = self.frames.read();
            frames.len() as i64 + 1
        };

        let frame = WsFrame {
            id: frame_id,
            connection_id: connection_id.clone(),
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            direction,
            frame_type,
            payload,
            masked,
        };

        let mut frames = self.frames.write();
        frames.push(frame);

        // P4: Enforce global max frames limit (FIFO eviction)
        if frames.len() > self.max_frames {
            tracing::debug!(
                total_frames = frames.len(),
                max_frames = self.max_frames,
                "Global frame limit reached - evicting oldest frame"
            );
            frames.remove(0);
        }

        // P4: If per-connection limit exceeded, remove oldest frame from this connection
        let connection_frame_count = frames.iter().filter(|f| f.connection_id == connection_id).count();
        if connection_frame_count > self.max_frames_per_connection {
            if let Some(pos) = frames.iter().position(|f| f.connection_id == connection_id) {
                frames.remove(pos);
                tracing::debug!(
                    connection_id = %connection_id,
                    "Per-connection frame limit reached - evicted oldest frame"
                );
            }
        }

        // Update connection frame count
        let mut connections = self.connections.write();
        if let Some(conn) = connections.iter_mut().find(|c| c.id == connection_id) {
            conn.frames_count += 1;
        }

        Ok(frame_id)
    }

    /// Get all connections
    pub fn get_connections(&self) -> Vec<WsConnection> {
        self.connections.read().clone()
    }

    /// Get frames for a specific connection
    pub fn get_frames(&self, connection_id: &str) -> Vec<WsFrame> {
        let frames = self.frames.read();
        frames
            .iter()
            .filter(|f| f.connection_id == connection_id)
            .cloned()
            .collect()
    }

    /// Get all frames
    pub fn get_all_frames(&self) -> Vec<WsFrame> {
        self.frames.read().clone()
    }

    /// Clear all captured data
    pub fn clear(&self) {
        self.connections.write().clear();
        self.frames.write().clear();
    }

    /// P4: Get memory usage statistics
    pub fn get_memory_stats(&self) -> WsMemoryStats {
        let frames = self.frames.read();
        let total_payload_bytes: usize = frames.iter().map(|f| f.payload.len()).sum();
        
        let connections = self.connections.read();
        let mut per_connection_stats = Vec::new();
        
        for conn in connections.iter() {
            let conn_frames = frames.iter().filter(|f| f.connection_id == conn.id).collect::<Vec<_>>();
            let conn_payload_bytes: usize = conn_frames.iter().map(|f| f.payload.len()).sum();
            per_connection_stats.push(ConnectionMemoryStats {
                connection_id: conn.id.clone(),
                frame_count: conn_frames.len(),
                payload_bytes: conn_payload_bytes,
                limit_percentage: (conn_frames.len() as f64 / self.max_frames_per_connection as f64) * 100.0,
            });
        }

        WsMemoryStats {
            total_frames: frames.len(),
            max_frames: self.max_frames,
            total_payload_bytes,
            total_connections: connections.len(),
            per_connection: per_connection_stats,
            global_limit_percentage: (frames.len() as f64 / self.max_frames as f64) * 100.0,
        }
    }

    /// P4: Check if memory usage is critical (>80% of limit)
    pub fn is_memory_critical(&self) -> bool {
        let frames = self.frames.read();
        (frames.len() as f64 / self.max_frames as f64) > 0.8
    }
}

impl Default for WsCapture {
    fn default() -> Self {
        Self::new(10_000) // Default: 10k frames
    }
}

/// P4: WebSocket memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMemoryStats {
    pub total_frames: usize,
    pub max_frames: usize,
    pub total_payload_bytes: usize,
    pub total_connections: usize,
    pub per_connection: Vec<ConnectionMemoryStats>,
    pub global_limit_percentage: f64,
}

/// P4: Per-connection memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMemoryStats {
    pub connection_id: String,
    pub frame_count: usize,
    pub payload_bytes: usize,
    pub limit_percentage: f64,
}

/// WebSocket frame parser/builder
pub struct WsFrameParser;

impl WsFrameParser {
    /// Parse WebSocket frame type from opcode
    pub fn parse_opcode(opcode: u8) -> Option<WsFrameType> {
        match opcode {
            0x1 => Some(WsFrameType::Text),
            0x2 => Some(WsFrameType::Binary),
            0x9 => Some(WsFrameType::Ping),
            0xA => Some(WsFrameType::Pong),
            0x8 => Some(WsFrameType::Close),
            _ => None,
        }
    }

    /// Convert frame type to opcode
    pub fn to_opcode(frame_type: &WsFrameType) -> u8 {
        match frame_type {
            WsFrameType::Text => 0x1,
            WsFrameType::Binary => 0x2,
            WsFrameType::Ping => 0x9,
            WsFrameType::Pong => 0xA,
            WsFrameType::Close => 0x8,
        }
    }

    /// Unmask WebSocket payload
    pub fn unmask_payload(payload: &[u8], mask: &[u8; 4]) -> Vec<u8> {
        payload
            .iter()
            .enumerate()
            .map(|(i, byte)| byte ^ mask[i % 4])
            .collect()
    }

    /// Mask WebSocket payload
    pub fn mask_payload(payload: &[u8], mask: &[u8; 4]) -> Vec<u8> {
        Self::unmask_payload(payload, mask) // XOR is symmetric
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_capture() {
        let capture = WsCapture::new(100);

        capture.register_connection("conn1".to_string(), "wss://example.com/ws".to_string());

        let frame_id = capture.capture_frame(
            "conn1".to_string(),
            WsDirection::ClientToServer,
            WsFrameType::Text,
            b"Hello".to_vec(),
            true,
        ).expect("capture_frame should succeed");

        assert_eq!(frame_id, 1);

        let frames = capture.get_frames("conn1");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].payload, b"Hello");
    }

    #[test]
    fn test_ws_memory_limits() {
        let capture = WsCapture::with_limits(100, 10);
        capture.register_connection("conn1".to_string(), "wss://example.com/ws".to_string());

        // Fill up to limit
        for i in 0..10 {
            let payload = format!("frame{}", i).into_bytes();
            capture.capture_frame(
                "conn1".to_string(),
                WsDirection::ClientToServer,
                WsFrameType::Text,
                payload,
                true,
            ).expect("capture_frame should succeed");
        }

        // At limit, next frame should still succeed (FIFO eviction)
        capture.capture_frame(
            "conn1".to_string(),
            WsDirection::ClientToServer,
            WsFrameType::Text,
            b"overflow".to_vec(),
            true,
        ).expect("capture_frame should succeed with eviction");

        // Should still have 10 frames (oldest was removed)
        let frames = capture.get_frames("conn1");
        assert_eq!(frames.len(), 10);
    }

    #[test]
    fn test_ws_payload_size_limit() {
        let capture = WsCapture::new(100);
        capture.register_connection("conn1".to_string(), "wss://example.com/ws".to_string());

        // Try to send payload larger than limit
        let oversized_payload = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        let result = capture.capture_frame(
            "conn1".to_string(),
            WsDirection::ClientToServer,
            WsFrameType::Binary,
            oversized_payload,
            false,
        );

        assert!(result.is_err(), "Oversized payload should be rejected");
    }

    #[test]
    fn test_unmask() {
        let payload = vec![0x7f, 0x9f, 0x4d, 0x51, 0x58];
        let mask = [0x37, 0xfa, 0x21, 0x3d];
        let unmasked = WsFrameParser::unmask_payload(&payload, &mask);

        // Verify unmasking works
        assert_ne!(unmasked, payload);

        // Verify masking is symmetric
        let remasked = WsFrameParser::mask_payload(&unmasked, &mask);
        assert_eq!(remasked, payload);
    }
}
