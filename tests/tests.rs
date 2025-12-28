use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::Once;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use zero2prod::configuration::*;

static TRACING: Once = Once::new();

struct TestApp {
    address: String,
    pool: PgPool,
}

async fn run_app() -> TestApp {
    // Init tracing
    TRACING.call_once(|| {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "zero2prod=debug,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    });

    // Run App
    let addr = "0.0.0.0:0";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let pool = configure_database(&configuration.database).await;

    let _ = tokio::spawn(zero2prod::run(listener, pool.clone()));

    let address = format!("http://localhost:{}", port);
    TestApp { address, pool }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: "password".to_string(),
        ..config.clone()
    };

    let mut connection = PgConnection::connect(&maintenance_settings.connection_string())
        .await
        .unwrap();

    connection.execute(format!(r#"CREATE DATABASE "{}""#, config.database_name).as_str())
        .await
        .unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(&maintenance_settings.connection_string())
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    pool
}

#[tokio::test]
async fn test_health_check() {
    // Start App
    let app = run_app().await;
    // Start client
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    println!("{:?}", response);

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn valid_user_subscribe_returns_200() {
    let app = run_app().await;
    let client = reqwest::Client::new();

    // POST request body of a valid user subscribing
    let body = "name=lupicipro&email=asdlolazoasd%40gmail.com"; // %40 == @ in url encoded
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        200,
        response.status().as_u16(),
        "unsuccessfull request, address: {}",
        app.address
    );

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "asdlolazoasd@gmail.com");
    assert_eq!(saved.name, "lupicipro");
}

#[tokio::test]
async fn invalid_user_subscribe_returns_400() {
    let app = run_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=lupicipro", "no email"),
        ("email=asdlolazoasd%40gmail.com", "no name"),
        ("", "missing name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was: {}, address: {}",
            error_message,
            app.address
        );
    }
}
