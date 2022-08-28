pub mod application;
pub mod frontend_api;
pub mod frontend_serve;
pub mod node_api;

use lazy_static::lazy_static;
use shared_utils::{DataTypeMarker, NodeConfig};
use std::collections::HashSet;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Configure the backend server here
static PG_CONNECTION_STRING: &str = "postgres://postgres:password@localhost:5432/backend";
lazy_static! {
    static ref DEFAULT_NODE_CONFIG: NodeConfig = {
        let mut data_types = HashSet::new();
        data_types.insert(DataTypeMarker::Fft);
        NodeConfig {
            data_types,
            freq: 1000000,
            amp: 1,
            lna: 0,
            vga: 0,
            sample_rate: 4000000,
        }
    };
    static ref BIND_ADDR: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3000));
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=trace,tower_http=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    application::start_up().await
}
