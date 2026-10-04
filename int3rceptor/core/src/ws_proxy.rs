//! Bridge WebSocket frames the HTTP proxy has already upgraded.
//!
//! Frames are parsed only so they can be copied unchanged and recorded in
//! [`WsCapture`](crate::websocket::WsCapture). Payload bytes stored for the UI
//! are unmasked; the bytes written to the peer are the original frame.

use crate::websocket::{WsCapture, WsDirection, WsFrameParser, WsFrameType};
use hyper::header::{HeaderMap, CONNECTION, UPGRADE};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// True when this HTTP request is asking to switch to WebSocket.
pub fn is_websocket_upgrade(headers: &HeaderMap) -> bool {
    let upgrade = headers
        .get(UPGRADE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase().contains("websocket"))
        .unwrap_or(false);
    if !upgrade {
        return false;
    }
    headers
        .get(CONNECTION)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase().contains("upgrade"))
        .unwrap_or(false)
}

/// Display URL for a captured WebSocket (`ws://` or `wss://`).
pub fn display_url(uri: &hyper::Uri) -> String {
    let scheme = match uri.scheme_str() {
        Some("https" | "wss") => "wss",
        _ => "ws",
    };
    let auth = uri.authority().map(|a| a.as_str()).unwrap_or("");
    let path = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");
    format!("{scheme}://{auth}{path}")
}

pub(crate) struct ParsedFrame {
    pub(crate) raw: Vec<u8>,
    pub(crate) opcode: u8,
    pub(crate) payload: Vec<u8>,
    pub(crate) masked: bool,
}

fn opcode_to_type(opcode: u8) -> WsFrameType {
    WsFrameParser::parse_opcode(opcode).unwrap_or(WsFrameType::Binary)
}

async fn read_exact_n<R: AsyncRead + Unpin>(reader: &mut R, n: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}

/// Read one WebSocket frame. The returned payload is unmasked.
pub(crate) async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> std::io::Result<ParsedFrame> {
    let header = read_exact_n(reader, 2).await?;
    let opcode = header[0] & 0x0f;
    let masked = header[1] & 0x80 != 0;
    let mut len = (header[1] & 0x7f) as u64;
    let mut raw = header;

    if len == 126 {
        let ext = read_exact_n(reader, 2).await?;
        len = u16::from_be_bytes([ext[0], ext[1]]) as u64;
        raw.extend_from_slice(&ext);
    } else if len == 127 {
        let ext = read_exact_n(reader, 8).await?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&ext);
        len = u64::from_be_bytes(arr);
        raw.extend_from_slice(&ext);
    }

    if len > crate::websocket::MAX_PAYLOAD_SIZE as u64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "websocket frame exceeds capture limit",
        ));
    }

    let mut mask = [0u8; 4];
    if masked {
        let mask_bytes = read_exact_n(reader, 4).await?;
        mask.copy_from_slice(&mask_bytes);
        raw.extend_from_slice(&mask_bytes);
    }

    let payload_raw = read_exact_n(reader, len as usize).await?;
    raw.extend_from_slice(&payload_raw);
    let payload = if masked {
        WsFrameParser::unmask_payload(&payload_raw, &mask)
    } else {
        payload_raw
    };

    Ok(ParsedFrame {
        raw,
        opcode,
        payload,
        masked,
    })
}

async fn pump<R, W>(
    mut reader: R,
    mut writer: W,
    capture: Option<Arc<WsCapture>>,
    connection_id: String,
    direction: WsDirection,
) where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    loop {
        let frame = match read_frame(&mut reader).await {
            Ok(frame) => frame,
            Err(_) => break,
        };
        if writer.write_all(&frame.raw).await.is_err() {
            break;
        }
        let _ = writer.flush().await;
        if let Some(capture) = &capture {
            let _ = capture.capture_frame(
                connection_id.clone(),
                direction.clone(),
                opcode_to_type(frame.opcode),
                frame.payload,
                frame.masked,
            );
        }
        if frame.opcode == 0x8 {
            break;
        }
    }
}

/// Copy frames both ways. `capture` is `None` when the request is out of scope.
pub async fn relay(
    client: Upgraded,
    server: Upgraded,
    capture: Option<Arc<WsCapture>>,
    connection_id: String,
) {
    let client = TokioIo::new(client);
    let server = TokioIo::new(server);
    let (client_read, client_write) = tokio::io::split(client);
    let (server_read, server_write) = tokio::io::split(server);

    let to_server = pump(
        client_read,
        server_write,
        capture.clone(),
        connection_id.clone(),
        WsDirection::ClientToServer,
    );
    let to_client = pump(
        server_read,
        client_write,
        capture,
        connection_id,
        WsDirection::ServerToClient,
    );
    tokio::join!(to_server, to_client);
}
