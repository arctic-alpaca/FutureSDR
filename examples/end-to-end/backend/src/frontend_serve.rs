use crate::application::{handle_file_serve_error, DataTypeMarker, NodeId, State};
use axum::body::Body;
use axum::extract::Path;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get_service;
use axum::{Extension, Router};
use std::sync::Arc;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

/// Only serve if the data stream exists, otherwise return 404.
pub async fn serve_data_visualizer(
    Extension(state): Extension<Arc<State>>,
    Path((node_id, data_type)): Path<(NodeId, DataTypeMarker)>,
    request: Request<Body>,
) -> impl IntoResponse {
    {
        let state_lock = state.nodes.lock().await;
        if let Some(node_state) = state_lock.get(&node_id) {
            let node_state_lock = node_state.data_streams.lock().await;
            if !node_state_lock.contains_key(&data_type) {
                return StatusCode::NOT_FOUND.into_response();
            }
        } else {
            return StatusCode::NOT_FOUND.into_response();
        }
    }

    let file_server_service = ServeDir::new("serve/frontend");
    let router = Router::new()
        .nest(
            "/frontend/:node_id/:data_type",
            get_service(file_server_service).handle_error(handle_file_serve_error),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
    router.oneshot(request).await.unwrap()
}
