use axum::http::{HeaderValue, StatusCode};
use axum::routing::get_service;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use sqlx::PgPool;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
pub struct State {
    pub db_pool: PgPool,
    pub data: Arc<Mutex<Option<Vec<u8>>>>,
    pub notifier: Arc<Mutex<Sender<bool>>>,
    pub receiver: Arc<Receiver<bool>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "end_to_end_backend=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (sender, receiver) = broadcast::channel(10);
    let db_pool = PgPool::connect("postgres://postgres:password@localhost:5432/backend")
        .await
        .expect("Failed to create database connection");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate database");

    let state = Arc::new(State {
        db_pool,
        data: Arc::new(Mutex::new(Some(vec![]))),
        notifier: Arc::new(Mutex::new(sender)),
        receiver: Arc::new(receiver),
    });

    let cors = CorsLayer::new().allow_origin(Any);
    let coep_header = SetResponseHeaderLayer::if_not_present(
        headers::HeaderName::from_str("cross-origin-embedder-policy").unwrap(),
        HeaderValue::from_static("require-corp"),
    );
    let coop_header = SetResponseHeaderLayer::if_not_present(
        headers::HeaderName::from_str("cross-origin-opener-policy").unwrap(),
        HeaderValue::from_static("same-origin"),
    );

    let file_server_service = ServeDir::new("serve");

    let app = Router::new()
        .route("/ws/node", get(ws_handler))
        .route("/ws/frontend", get(frontend_handler))
        .fallback(get_service(file_server_service).handle_error(handle_file_serve_error))
        .layer(Extension(state))
        .layer(cors)
        .layer(coep_header)
        .layer(coop_header)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_file_serve_error(err: io::Error) -> impl IntoResponse {
    debug!(%err);
    (StatusCode::INTERNAL_SERVER_ERROR, "Failure to serve file")
}

async fn ws_handler(
    Extension(state): Extension<Arc<State>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    debug!("node connected");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<State>) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    debug!("node client sent str: {:?}", t);
                }
                Message::Binary(data) => {
                    {
                        let mut lock = state.data.lock().await;
                        sqlx::query!("INSERT INTO data_storage (data) VALUES ($1)", data)
                            .execute(&state.db_pool)
                            .await
                            .expect("Failed to insert data into database");
                        *lock = Some(data);
                    }
                    {
                        let lock = state.notifier.lock().await;
                        lock.send(true).unwrap();
                    }
                    debug!("node sent data");
                }
                Message::Ping(_) => {
                    debug!("node socket ping");
                }
                Message::Pong(_) => {
                    debug!("node socket pong");
                }
                Message::Close(_) => {
                    debug!("node client disconnected");
                    return;
                }
            }
        } else {
            debug!("node client disconnected");
            return;
        }
    }
}

async fn frontend_handler(
    Extension(state): Extension<Arc<State>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    debug!("frontend connected");
    ws.on_upgrade(|socket| handle_frontend_socket(socket, state))
}

async fn handle_frontend_socket(mut socket: WebSocket, state: Arc<State>) {
    let mut x = { state.notifier.lock().await.subscribe() };
    while x.recv().await.is_ok() {
        let data = { state.data.lock().await.take() };
        if data.is_some() && socket.send(Message::Binary(data.unwrap())).await.is_err() {
            debug!("frontend client disconnected");
            return;
        }
    }
}
