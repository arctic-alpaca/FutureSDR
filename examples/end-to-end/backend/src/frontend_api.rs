use crate::application::{DataTypeMarker, NodeId, State};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{response::IntoResponse, Extension};
use axum_tungstenite::{Message, WebSocket, WebSocketUpgrade};
use futures::SinkExt;

use std::sync::Arc;
use tokio::sync::broadcast;

use tracing::{debug, error, trace};

pub async fn frontend_data_ws_handler(
    Extension(state): Extension<Arc<State>>,
    Path((node_id, data_type)): Path<(NodeId, DataTypeMarker)>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    debug!("frontend connected");

    let receiver = {
        let nodes_lock = state.nodes.lock().await;
        if let Some(node) = nodes_lock.get(&node_id) {
            let data_stream_lock = node.data_streams.lock().await;
            if let Some(sender) = data_stream_lock.get(&data_type) {
                sender.subscribe()
            } else {
                //TODO: Send command to node, create stream, wait?
                return StatusCode::NOT_FOUND.into_response();
            }
        } else {
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    ws.on_upgrade(move |socket| handle_frontend_ws_binary_data(socket, receiver))
}

async fn handle_frontend_ws_binary_data(
    mut socket: WebSocket,
    mut receiver: broadcast::Receiver<Arc<Vec<u8>>>,
) {
    // Data is sent in chunks since the frontend only accepts input of 2048 f32.
    while let Ok(data) = receiver.recv().await {
        let chunks_vec: Vec<Result<Message, _>> = data
            .chunks(data.len() / 20)
            .map(|v| Ok(Message::Binary(v.to_vec())))
            .collect();
        let mut chunks_stream = futures::stream::iter(chunks_vec);
        // We need to use axum_tungestnite since send_all returns a tungstenite error.
        if let Err(e) = socket.send_all(&mut chunks_stream).await {
            match e {
                axum_tungstenite::Error::Io(e) => {
                    error!("Frontend websocket IO error: {}", e);
                    return;
                }
                axum_tungstenite::Error::AlreadyClosed => {
                    error!("Frontend websocket connection already closed");
                    return;
                }
                axum_tungstenite::Error::ConnectionClosed => {
                    trace!("Frontend websocket connection closed");
                    return;
                }
                axum_tungstenite::Error::Capacity(e) => {
                    error!("Frontend websocket connection capacity error {:?}", e);
                }
                axum_tungstenite::Error::Http(_) => {
                    error!("Frontend websocket connection http error");
                    return;
                }
                axum_tungstenite::Error::HttpFormat(e) => {
                    error!("Frontend websocket connection http format error {}", e);
                    return;
                }
                axum_tungstenite::Error::Protocol(e) => {
                    error!("Frontend websocket connection protocol error {:?}", e);
                    return;
                }
                axum_tungstenite::Error::SendQueueFull(e) => {
                    error!("Frontend websocket connection send queue full error {}", e);
                }
                axum_tungstenite::Error::Tls(e) => {
                    error!("Frontend websocket connection TLS error {:?}", e);
                    return;
                }
                axum_tungstenite::Error::Url(e) => {
                    error!("Frontend websocket connection url error {:?}", e);
                    return;
                }
                axum_tungstenite::Error::Utf8 => {
                    error!("Frontend websocket connection UTF8 error");
                    return;
                }
            }
        };
    }
}
