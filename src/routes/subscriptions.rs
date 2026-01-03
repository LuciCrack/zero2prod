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

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        subscriber_email=%data.email,
        subscriber_name=%data.name,
    )
)]
pub async fn subscribe(State(pool): State<PgPool>, Form(data): Form<FormData>) -> StatusCode {
    match insert_subscriber(&pool, &data).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Error in subscribing query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber to database"
    skip_all
)]
async fn insert_subscriber(pool: &PgPool, data: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        data.email,
        data.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Error in inserting subscription: {:?}", e);
        e
    })?;
    Ok(())
}
