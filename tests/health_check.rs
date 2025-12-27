#[tokio::test]
async fn test_health_check() {
    // Run App
    let addr = "0.0.0.0:0";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let _ = tokio::spawn(zero2prod::run(listener));

    // Start client
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://localhost:{}/health_check", port))
        .send().await
        .expect("Failed to execute request.");

    println!("{:?}", response);

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}