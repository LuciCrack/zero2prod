use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use uuid::Uuid;
use zero2prod::configuration::*;
use zero2prod::telemetry::{init_subscriber, get_subscriber};

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout,
        );
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink,
        );
        init_subscriber(subscriber);
    }
});

struct TestApp {
    address: String,
    pool: PgPool,
}

async fn run_app() -> TestApp {
    // Init tracing
    LazyLock::force(&TRACING);

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

    let connection_string= &maintenance_settings.connection_string();
    tracing::info!("{}", connection_string);

    let mut connection = PgConnection::connect(connection_string)
        .await
        .unwrap();

    (&mut connection)
        .execute(format!(r#"CREATE DATABASE "{}""#, config.database_name).as_str())
        .await
        .unwrap();

    // Create Pool Connection to new database
    let isolated_connection_string = &config.connection_string();

    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(isolated_connection_string)
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

    tracing::info!("health_check Response: {:?}", response);

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn valid_user_subscribe_returns_200() {
    let app = run_app().await;
    let client = reqwest::Client::new();

    // POST request body of a valid user subscribing
    let body = "name=username&email=random.user%40gmail.com"; // %40 == @ in url encoded
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

    assert_eq!(saved.email, "random.user@gmail.com");
    assert_eq!(saved.name, "username");
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
            "The API did not fail with 422 Bad Request when the payload was: {}, address: {}",
            error_message,
            app.address
        );
    }
}
