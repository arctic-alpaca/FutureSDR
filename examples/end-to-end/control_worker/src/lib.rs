mod ws_tasks;

use futures::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_worker::{HandlerId, Worker, WorkerScope};
use shared_utils::{FromCommandMsg, Msg, NodeToBackend, ToCommandMsg};
use wasm_bindgen_futures::spawn_local;

#[cfg(feature = "include_main")]
use gloo_worker::Registrable;
#[cfg(feature = "include_main")]
use wasm_bindgen::prelude::*;

/// Used to communicate from the worker to ws sender task.
#[derive(Debug)]
pub enum ToWsSenderTask {
    RequestConfig,
    AckConfig,
}

pub struct ControlWorker {
    ws_sender: futures::channel::mpsc::Sender<NodeToBackend>,
    ws_receiver: Option<futures::channel::mpsc::Receiver<NodeToBackend>>,
}

impl Worker for ControlWorker {
    type Message = Msg<()>;

    type Input = ToCommandMsg;

    type Output = FromCommandMsg;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        let (ws_sender, ws_receiver) = futures::channel::mpsc::channel(5);
        Self {
            ws_sender,
            ws_receiver: Some(ws_receiver),
        }
    }

    fn update(&mut self, _scope: &WorkerScope<Self>, _msg: Self::Message) {}

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, who: HandlerId) {
        match msg {
            ToCommandMsg::Initialize => {
                let scope_clone = scope.clone();
                let scope_clone2 = scope.clone();
                match WebSocket::open("ws://127.0.0.1:3000/node/api/control") {
                    Ok(ws) => {
                        let (write, read) = ws.split();

                        spawn_local(
                            async move { ws_tasks::run_receiver(read, scope_clone, who).await },
                        );
                        let receiver = self.ws_receiver.take().unwrap();
                        spawn_local(async move {
                            ws_tasks::run_sender(write, receiver, scope_clone2, who).await
                        });
                    }
                    Err(e) => scope.respond(
                        who,
                        FromCommandMsg::Terminate {
                            msg: format!("Failed to open WebSocket connection: {e}"),
                        },
                    ),
                }
            }
            ToCommandMsg::AckConfig { config } => {
                let mut sender_clone = self.ws_sender.clone();
                let scope_clone = scope.clone();
                spawn_local(async move {
                    if let Err(e) = sender_clone.send(NodeToBackend::AckConfig { config }).await {
                        scope_clone.respond(who, FromCommandMsg::Terminate { msg: e.to_string() })
                    }
                });
            }
            ToCommandMsg::GetInitialConfig => {
                let mut sender_clone = self.ws_sender.clone();
                let scope_clone = scope.clone();
                spawn_local(async move {
                    if let Err(e) = sender_clone.send(NodeToBackend::RequestConfig).await {
                        scope_clone.respond(who, FromCommandMsg::Terminate { msg: e.to_string() })
                    }
                });
            }
        }
    }
}

#[cfg(feature = "include_main")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    ControlWorker::registrar().register();
}
