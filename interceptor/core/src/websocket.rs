use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use time::OffsetDateTime;

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
#[derive(Clone)]
pub struct WsCapture {
    connections: Arc<RwLock<Vec<WsConnection>>>,
    frames: Arc<RwLock<Vec<WsFrame>>>,
    max_frames: usize,
}

impl WsCapture {
    pub fn new(max_frames: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            frames: Arc::new(RwLock::new(Vec::new())),
            max_frames,
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

        let mut connections = self.connections.write().unwrap();
        connections.push(connection);
    }

    /// Mark a connection as closed
    pub fn close_connection(&self, id: &str) {
        let mut connections = self.connections.write().unwrap();
        if let Some(conn) = connections.iter_mut().find(|c| c.id == id) {
            conn.closed_at = Some(OffsetDateTime::now_utc().unix_timestamp());
        }
    }

    /// Capture a WebSocket frame
    pub fn capture_frame(
        &self,
        connection_id: String,
        direction: WsDirection,
        frame_type: WsFrameType,
        payload: Vec<u8>,
        masked: bool,
    ) -> i64 {
        let frame_id = {
            let frames = self.frames.read().unwrap();
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

        let mut frames = self.frames.write().unwrap();
        frames.push(frame);

        // Enforce max frames limit (FIFO)
        if frames.len() > self.max_frames {
            frames.remove(0);
        }

        // Update connection frame count
        let mut connections = self.connections.write().unwrap();
        if let Some(conn) = connections.iter_mut().find(|c| c.id == connection_id) {
            conn.frames_count += 1;
        }

        frame_id
    }

    /// Get all connections
    pub fn get_connections(&self) -> Vec<WsConnection> {
        self.connections.read().unwrap().clone()
    }

    /// Get frames for a specific connection
    pub fn get_frames(&self, connection_id: &str) -> Vec<WsFrame> {
        let frames = self.frames.read().unwrap();
        frames
            .iter()
            .filter(|f| f.connection_id == connection_id)
            .cloned()
            .collect()
    }

    /// Get all frames
    pub fn get_all_frames(&self) -> Vec<WsFrame> {
        self.frames.read().unwrap().clone()
    }

    /// Clear all captured data
    pub fn clear(&self) {
        self.connections.write().unwrap().clear();
        self.frames.write().unwrap().clear();
    }
}

impl Default for WsCapture {
    fn default() -> Self {
        Self::new(10_000) // Default: 10k frames
    }
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
        );
        
        assert_eq!(frame_id, 1);
        
        let frames = capture.get_frames("conn1");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].payload, b"Hello");
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
