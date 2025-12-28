use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use zero2prod::configuration::get_configuration;
use zero2prod::run;

#[tokio::main]
async fn main() {
    // Load configuration from configuration.yaml
    let configuration = get_configuration().expect("Failed to read configuration");

    // Init database pool
    let pg_pool: PgPool = PgPoolOptions::new()
        .max_connections(100)
        .connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres pool");

    // Init tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zero2prod=trace,tower_http=error".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let address = format!("0.0.0.0:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let _ = run(listener, pg_pool).await;
}
