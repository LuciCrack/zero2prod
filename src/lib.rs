use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    // build our application with a single route
    let app = Router::new()
        .route("/health_check", get(health_check))
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 2000
    tracing::info!("App running on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await
}

async fn health_check() {
    // doing nothing gives code 200
}
