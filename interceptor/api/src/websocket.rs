use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Extension, WebSocketUpgrade};
use axum::response::IntoResponse;
use serde_json::json;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

use crate::state::AppState;
use interceptor_core::capture::CaptureEntry;
use std::sync::Arc;

pub async fn ws_route(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let receiver = state.capture.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, receiver))
}

async fn handle_socket(mut socket: WebSocket, mut receiver: broadcast::Receiver<CaptureEntry>) {
    let welcome = json!({
        "type": "hello",
        "data": "connected",
    });
    if socket
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        match receiver.recv().await {
            Ok(entry) => {
                let payload = json!({
                    "type": "request",
                    "data": entry,
                });
                if socket
                    .send(Message::Text(payload.to_string()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Err(RecvError::Lagged(_)) => continue,
            Err(RecvError::Closed) => break,
        }
    }
}
