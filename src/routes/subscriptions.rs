use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(State(pool): State<PgPool>, Form(data): Form<FormData>) -> StatusCode {
    let result = sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        data.email,
        data.name,
        Utc::now()
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Error in subscribing query: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
