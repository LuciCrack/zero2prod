use axum::{
    routing::get,
    Router,
    extract::Path,
};
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    // Init tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zero2prod=trace,tower_http=error".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with a single route
    let app = Router::new()
        .route("/hello", get(hello))
        .route("/hello/{name}", get(hello))
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2000").await.unwrap();
    tracing::info!("App running on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct HelloParams {
    name: Option<String>,
}

async fn hello(Path(params): Path<HelloParams>) -> String {
    if let Some(name) = params.name {
        format!("Hello, {}!", name)
    } else {
        "Hello, World!".to_string()
    }
}