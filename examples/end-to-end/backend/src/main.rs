use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Redirect;
use axum::routing::get_service;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};
use tower_cookies::{CookieManagerLayer, Cookies};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
pub struct State {
    pub db_pool: PgPool,
    pub nodes: Arc<Mutex<HashMap<NodeId, NodeState>>>,
}

pub enum NodeControlMsg {
    ChangeDataType { data_type: DataTypeMarker },
    AnnounceId { node_id: NodeId },
}

/// Used to mark what kind of data a node does or should produce. Also used in path parameter for node and frontend.
#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataTypeMarker {
    Fft,
    ZigBee,
}

type NodeId = uuid::Uuid;

#[derive(Debug)]
pub struct NodeState {
    pub data_streams: Arc<Mutex<HashMap<DataTypeMarker, Sender<Vec<u8>>>>>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[allow(dead_code)]
struct SdrParamter {
    #[serde(default)]
    freq: Option<u32>,
    #[serde(default)]
    amp: Option<u32>,
    #[serde(default)]
    lna: Option<u32>,
    #[serde(default)]
    vga: Option<u32>,
    #[serde(default)]
    sample_rate: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FrontendPathParameter {
    node_id: NodeId,
    mode: DataTypeMarker,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=trace,tower_http=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_pool = PgPool::connect("postgres://postgres:password@localhost:5432/backend")
        .await
        .expect("Failed to create database connection");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate database");

    let state = Arc::new(State {
        db_pool,
        nodes: Arc::new(Mutex::new(HashMap::new())),
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
        .route("/node/data/:data_type", get(node_data_ws_handler))
        .route("/ws/frontend", get(frontend_handler))
        .route(
            "/frontend_api/:node_id",
            get(|Path(node_id): Path<NodeId>| async move {
                Redirect::permanent(&format!("/frontend_api/{node_id}/fft"))
            }),
        )
        .fallback(get_service(file_server_service).handle_error(handle_file_serve_error))
        .layer(Extension(state))
        .layer(cors)
        .layer(coep_header)
        .layer(coop_header)
        .layer(CookieManagerLayer::new())
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

async fn node_data_ws_handler(
    Extension(state): Extension<Arc<State>>,
    Path(data_type): Path<DataTypeMarker>,
    cookies: Cookies,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    debug!("node connected");
    debug!(?data_type);
    debug!(?cookies);
    if let Some(node_id) = cookies.get("node_id") {
        debug!("Found cookie: {}", node_id);
        let uuid = uuid::Uuid::from_str(node_id.value()).unwrap();
        debug!(?uuid);
    } else {
        // TODO: Reject
    }
    let node_id = uuid::Uuid::from_str(cookies.get("node_id").unwrap().value()).unwrap();
    ws.on_upgrade(move |socket| handle_node_data_ws(socket, node_id, data_type, state))
}

async fn handle_node_data_ws(
    mut socket: WebSocket,
    node_id: NodeId,
    data_type: DataTypeMarker,
    state: Arc<State>,
) {
    let sender: Sender<Vec<u8>>;
    {
        let mut state_lock = state.nodes.lock().await;
        if let std::collections::hash_map::Entry::Vacant(e) = state_lock.entry(node_id) {
            let (sender1, _) = broadcast::channel(10);
            sender = sender1.clone();
            let mut hm = HashMap::new();

            hm.insert(data_type, sender1);

            e.insert(NodeState {
                data_streams: Arc::new(Mutex::new(hm)),
            });
        } else {
            let mut data_streams_lock = state_lock.get(&node_id).unwrap().data_streams.lock().await;
            let (sender1, _) = broadcast::channel(10);
            sender = sender1.clone();
            data_streams_lock.insert(data_type, sender1);
        }
    }
    let mut counter = 0;
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    debug!("node client sent str: {:?}", t);
                }
                Message::Binary(data) => {
                    {
                        sqlx::query!("INSERT INTO data_storage (data) VALUES ($1)", data)
                            .execute(&state.db_pool)
                            .await
                            .expect("Failed to insert data into database");
                    }
                    {
                        if sender.receiver_count() >= 1 {
                            sender.send(data).unwrap();
                        }
                    }
                    //TODO Debug, remove
                    counter += 1;
                    if counter >= 100 {
                        debug!("node sent data");
                        counter = 0;
                    }
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
    let mut x = {
        state
            .nodes
            .lock()
            .await
            .values()
            .next()
            .unwrap()
            .data_streams
            .lock()
            .await
            .values()
            .next()
            .unwrap()
            .subscribe()
    };
    while let Ok(data) = x.recv().await {
        if socket.send(Message::Binary(data)).await.is_err() {
            debug!("frontend client disconnected");
            return;
        }
    }
}
