use crate::application::{DataTypeMarker, NodeId, NodeSdrParameters, NodeState, State};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{response::IntoResponse, Extension};
use axum_tungstenite::{Message, WebSocket, WebSocketUpgrade};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};
use tower_cookies::Cookies;
use tracing::{debug, error};

pub async fn node_data_ws_handler(
    Extension(state): Extension<Arc<State>>,
    Path(node_sdr_parameters): Path<NodeSdrParameters>,
    cookies: Cookies,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    debug!("node connected");

    let node_id = if let Some(node_id) = cookies.get("node_id") {
        if let Ok(uuid) = uuid::Uuid::from_str(node_id.value()) {
            NodeId(uuid)
        } else {
            return StatusCode::BAD_REQUEST.into_response();
        }
    } else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    ws.on_upgrade(move |socket| {
        handle_node_data_ws(socket, node_id, node_sdr_parameters.data_type, state)
    })
}

async fn handle_node_data_ws(
    mut socket: WebSocket,
    node_id: NodeId,
    data_type: DataTypeMarker,
    state: Arc<State>,
) {
    let sender = {
        let mut state_lock = state.nodes.lock().await;

        match state_lock.entry(node_id) {
            Entry::Occupied(entry) => {
                let mut data_streams_lock = entry.get().data_streams.lock().await;
                let (sender, _) = broadcast::channel(10);
                data_streams_lock.insert(data_type, sender.clone());
                sender
            }
            Entry::Vacant(entry) => {
                let (sender, _) = broadcast::channel(10);
                let mut data_streams_hashmap = HashMap::new();

                data_streams_hashmap.insert(data_type, sender.clone());

                entry.insert(NodeState {
                    data_streams: Arc::new(Mutex::new(data_streams_hashmap)),
                });
                sender
            }
        }
    };

    // Counter to reduce the amount
    #[cfg(debug_assertions)]
    let mut counter = 0;

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Binary(data) => {
                    {
                        sqlx::query!("INSERT INTO data_storage (data) VALUES ($1)", data)
                            .execute(&state.db_pool)
                            .await
                            .expect("Failed to insert data into database");
                    }

                    let data = Arc::new(data);
                    if sender.receiver_count() >= 1 {
                        // According to tokio docs: A send operation can only fail if there are no
                        // active receivers, implying that the message could never be received.
                        // The error contains the message being sent as a payload so it can be
                        // recovered.
                        sender
                            .send(data)
                            .expect("Broadcast channel send failed. This can't happen.");
                    }

                    #[cfg(debug_assertions)]
                    {
                        counter += 1;
                        if counter >= 1 {
                            debug!("node sent data: {}", node_id);
                            counter = 0;
                        }
                    }
                }
                Message::Close(_) => {
                    debug!("node client disconnected: {}", node_id);
                    node_cleanup(state, node_id).await;
                    return;
                }
                _ => {
                    error!(
                        "Node sent non-binary websocket message: {} - {:?}",
                        node_id, msg
                    );
                }
            }
        } else {
            node_cleanup(state, node_id).await;
            debug!("node client disconnected unexpectedly: {}", node_id);
            return;
        }
    }
}

/// Removes the node from the [State].
async fn node_cleanup(state: Arc<State>, node_id: NodeId) {
    //TODO add last seen to db
    //TODO currently hangs if the webpage get's F5ed, combine with new node handling
    let mut state_lock = state.nodes.lock().await;
    state_lock.remove(&node_id);
}
