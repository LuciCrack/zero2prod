use sqlx::postgres::{PgPool, PgPoolOptions};
use zero2prod::configuration::get_configuration;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
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

    // Init tracing subscriber
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    let address = format!("0.0.0.0:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let _ = run(listener, pg_pool).await;
}
