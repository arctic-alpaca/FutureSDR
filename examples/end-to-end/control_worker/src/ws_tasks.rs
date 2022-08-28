use crate::ControlWorker;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use gloo_worker::{HandlerId, WorkerScope};
use shared_utils::{BackendToNode, FromCommandMsg, NodeToBackend};

pub async fn run_receiver(
    mut receiver: SplitStream<WebSocket>,
    scope: WorkerScope<ControlWorker>,
    handler_id: HandlerId,
) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(s)) => scope.respond(
                handler_id,
                FromCommandMsg::Terminate {
                    msg: format!("Control worker: Received unexpected text message {s}"),
                },
            ),
            Ok(Message::Bytes(b)) => match bincode::deserialize::<BackendToNode>(&b) {
                Ok(BackendToNode::SendConfig { config }) => {
                    scope.respond(handler_id, FromCommandMsg::ReceivedConfig { config })
                }
                Err(e) => scope.respond(
                    handler_id,
                    FromCommandMsg::PrintToScreen {
                        msg: format!("Control worker: Deserializing message failed {e}"),
                    },
                ),
                Ok(BackendToNode::Error { msg, terminate }) => {
                    if terminate {
                        scope.respond(
                            handler_id,
                            FromCommandMsg::Terminate {
                                msg: format!("Backend error, terminating: {msg}"),
                            },
                        );
                    } else {
                        scope.respond(
                            handler_id,
                            FromCommandMsg::PrintToScreen {
                                msg: format!("Backend error, terminating: {msg}"),
                            },
                        );
                    }
                }
            },
            Err(_) => scope.respond(handler_id, FromCommandMsg::Disconnected {}),
        }
    }
}

pub async fn run_sender(
    mut sender: SplitSink<WebSocket, Message>,
    mut receiver: futures::channel::mpsc::Receiver<NodeToBackend>,
    scope: WorkerScope<ControlWorker>,
    handler_id: HandlerId,
) {
    while let Some(msg) = receiver.next().await {
        let encoded: Vec<u8> = bincode::serialize(&msg).unwrap();
        if let Err(e) = sender.send(Message::Bytes(encoded)).await {
            scope.respond(
                handler_id,
                FromCommandMsg::PrintToScreen {
                    msg: format!("WebSocket send failed: {e}"),
                },
            );
        }
    }
}
