use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::subscriber::set_global_default;
use tracing_subscriber::layer::SubscriberExt;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry};
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

    // Init Tracing
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "zero2prod=trace,tower_http=warn".into());

    let formatting_layer = BunyanFormattingLayer::new(
        "zero2prod".into(),
        std::io::stdout
    );

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

    let address = format!("0.0.0.0:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let _ = run(listener, pg_pool).await;
}
