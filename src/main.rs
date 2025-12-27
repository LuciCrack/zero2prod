use zero2prod::run;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2000").await.unwrap();
    run(listener).await;
}