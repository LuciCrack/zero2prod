use std::sync::Once;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static TRACING: Once = Once::new();

fn run_app() -> String {
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

    // Little workaround: I don't want this function to be async, to return a Future
    // So we will get a std TcpListener and turn it into a tokio::net::TcpListener
    let std_listener = std::net::TcpListener::bind(addr).unwrap();
    let _ = std_listener.set_nonblocking(true);
    let port = std_listener.local_addr().unwrap().port();
    let listener = tokio::net::TcpListener::from_std(std_listener).unwrap();
    let _ = tokio::spawn(zero2prod::run(listener));

    // Return the used address so the test can use it
    format!("http://localhost:{}", port)
}

#[tokio::test]
async fn test_health_check() {
    // Start App
    let addr = run_app();
    // Start client
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", addr))
        .send()
        .await
        .expect("Failed to execute request.");

    println!("{:?}", response);

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn valid_user_subscribe_returns_200() {
    let addr = run_app();
    let client = reqwest::Client::new();

    // POST request body of a valid user subscribing
    let body = "name=lupicipro&email=asdlolazoasd%40gmail.com"; // %40 == @ in url encoded
    let response = client
        .post(format!("{}/subscriptions", addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        200,
        response.status().as_u16(),
        "unsuccessfull request, address: {}",
        addr
    );
}

#[tokio::test]
async fn invalid_user_subscribe_returns_400() {
    let addr = run_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=lupicipro", "no email"),
        ("email=asdlolazoasd%40gmail.com", "no name"),
        ("", "missing name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was: {}, address: {}",
            error_message,
            addr
        );
    }
}
