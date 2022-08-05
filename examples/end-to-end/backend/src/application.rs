use crate::frontend_api::frontend_data_ws_handler;
use crate::frontend_serve::serve_data_visualizer;
use crate::node_api::node_data_ws_handler;
use crate::PG_CONNECTION_STRING;
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get_service;
use axum::{routing::get, Extension, Router};
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::debug;

#[derive(Debug)]
pub struct State {
    pub db_pool: PgPool,
    pub nodes: Arc<Mutex<HashMap<NodeId, NodeState>>>,
}

/// Used to mark what kind of data a node does or should produce. Also used in path parameter for node and frontend.
#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataTypeMarker {
    Fft,
    ZigBee,
}

/// New type for [uuid::Uuid].
#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Copy, Clone)]
pub struct NodeId(pub uuid::Uuid);

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type alias to make clippy happy (https://rust-lang.github.io/rust-clippy/master/index.html#type_complexity)
pub type DataStreamStorage = Arc<Mutex<HashMap<DataTypeMarker, Sender<Arc<Vec<u8>>>>>>;

#[derive(Debug)]
pub struct NodeState {
    pub data_streams: DataStreamStorage,
}

/// SDR parameters used to collect parameters for incoming node data from the request path.
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeSdrParameters {
    // default node config taken from https://github.com/bastibl/webusb-libusb/blob/works/example/hackrf_open.cc#L161
    // steps and ranges taken from https://hackrf.readthedocs.io/en/latest/faq.html#what-gain-controls-are-provided-by-hackrf
    // Sample rate and frequency taken from https://hackrf.readthedocs.io/en/latest/hackrf_one.html
    pub data_type: DataTypeMarker,

    // default: 2480000000 (2,48GHz)
    // 1MHz to 6 GHz (1000000 - 6000000000)
    pub freq: u64,
    // default 1
    // on or off (0 or 1)
    pub amp: u8,
    // default: 32
    // 0-40 in steps of 8
    pub lna: u8,
    // default: 14
    // 0-62 in steps of 2
    pub vga: u8,
    // default: 4000000 (4 Msps)
    // 1 Msps to 20 Msps (million samples per second) (1000000 - 20000000)
    pub sample_rate: u64,
}

/// NodeId and DataTypeMarker from the request path. Used to identify which node and what data stream
/// is requested.
#[derive(Debug, Deserialize, Serialize)]
struct FrontendPathParameter {
    node_id: NodeId,
    mode: DataTypeMarker,
}

pub async fn handle_file_serve_error(err: std::io::Error) -> impl IntoResponse {
    debug!(%err);
    (StatusCode::INTERNAL_SERVER_ERROR, "Failure to serve file")
}

async fn build_router() -> Router {
    let db_pool = PgPool::connect(PG_CONNECTION_STRING)
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

    Router::new()
        // node api
        .route(
            "/node_api/data/:data_type/:freq/:amp/:lna/:vga/:sample_rate",
            get(node_data_ws_handler),
        )
        // frontend api
        .route(
            "/frontend_api/data/:node_id/:data_type",
            get(frontend_data_ws_handler),
        )
        // serve data visualizer
        .route(
            "/frontend/:node_id/:data_type/*rest",
            get(serve_data_visualizer),
        )
        // File serving TODO specify file serving properly
        .fallback(get_service(file_server_service).handle_error(handle_file_serve_error))
        .layer(Extension(state))
        .layer(cors)
        .layer(coep_header)
        .layer(coop_header)
        .layer(CookieManagerLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
}

pub async fn start_up() {
    let app = build_router().await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
