use crate::application::handle_file_serve_error;
use axum::body::Body;
use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::get_service;
use axum::Router;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub async fn serve_data_visualizer(request: Request<Body>) -> impl IntoResponse {
    // We use this method to allow serving directly from disk with `ServeDir` while still allowing
    // path variables.
    let fft_server_service = ServeDir::new("serve/frontend/fft_visualizer");
    let zigbee_server_service = ServeDir::new("serve/frontend/zigbee_visualizer");
    let router = Router::new()
        .nest(
            "/frontend/fft_visualizer/:node_id/:data_type",
            get_service(fft_server_service.clone()).handle_error(handle_file_serve_error),
        )
        .nest(
            "/frontend/zigbee_visualizer/:node_id/:data_type",
            get_service(zigbee_server_service).handle_error(handle_file_serve_error),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
    router.oneshot(request).await.unwrap()
}
