use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::routes::{health_check, subscribe};
use crate::telemetry::create_subscriber_middleware_layer;

pub async fn run(listener: TcpListener, pool: PgPool) -> Result<(), std::io::Error> {
    let middleware_layer = create_subscriber_middleware_layer();
    // Build application
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(pool)
        .layer(middleware_layer);

    // run our app with hyper, listening globally on port 2000
    tracing::info!("App running on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await
}
