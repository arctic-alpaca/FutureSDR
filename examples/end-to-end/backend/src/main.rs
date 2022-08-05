pub mod application;
pub mod frontend_api;
pub mod frontend_serve;
pub mod node_api;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static PG_CONNECTION_STRING: &str = "postgres://postgres:password@localhost:5432/backend";

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
