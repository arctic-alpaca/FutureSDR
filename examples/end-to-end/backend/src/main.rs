// Example code taken from: https://github.com/tokio-rs/axum/tree/main/examples/websockets
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex, RwLock};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct State {
    pub data: Arc<RwLock<Vec<u8>>>,
    pub notifier: Arc<Mutex<Sender<bool>>>,
    pub receiver: Arc<Receiver<bool>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (sender, receiver) = broadcast::channel(10);
    let state = Arc::new(State {
        data: Arc::new(RwLock::new(vec![])),
        notifier: Arc::new(Mutex::new(sender)),
        receiver: Arc::new(receiver),
    });
    // build our application with some routes
    let app = Router::new()
        // routes are matched from bottom to top, so we have to put `nest` at the
        // top since it matches all routes
        .route("/ws", get(ws_handler))
        .route("/frontend", get(frontend_handler))
        // logging so we can see whats going on
        .layer(Extension(state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(
    Extension(state): Extension<Arc<State>>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<State>) {
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    println!("node client sent str: {:?}", t);
                }
                Message::Binary(data) => {
                    {
                        let mut lock = state.data.write().await;
                        *lock = data;
                    }
                    {
                        let lock = state.notifier.lock().await;
                        lock.send(true).unwrap();
                    }
                    println!("node client sent binary data");
                }
                Message::Ping(_) => {
                    println!("node socket ping");
                }
                Message::Pong(_) => {
                    println!("node socket pong");
                }
                Message::Close(_) => {
                    println!("node client disconnected");
                    return;
                }
            }
        } else {
            println!("node client disconnected");
            return;
        }
    }

    loop {
        if socket
            .send(Message::Text(String::from("Hi!")))
            .await
            .is_err()
        {
            println!("node client disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn frontend_handler(
    Extension(state): Extension<Arc<State>>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(|socket| handle_frontend_socket(socket, state))
}

async fn handle_frontend_socket(mut socket: WebSocket, state: Arc<State>) {
    let mut x = { state.notifier.lock().await.subscribe() };
    while x.recv().await.is_ok() {
        let data = { state.data.read().await.clone() };
        if socket.send(Message::Binary(data)).await.is_err() {
            println!("frontend client disconnected");
            return;
        }
    }
}
