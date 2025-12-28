use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::routes::{health_check, subscribe};

pub async fn run(listener: TcpListener, pool: PgPool) -> Result<(), std::io::Error> {
    // build our application with a single route
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(pool)
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 2000
    tracing::info!("App running on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await
}
