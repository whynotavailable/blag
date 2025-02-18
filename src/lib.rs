use crate::models::AppState;
use crate::routes::collect_routes;
use sqlx::postgres::PgPoolOptions;

mod models;
mod routes;

pub async fn setup() {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect("postgres://postgres:garden@localhost/handoff")
        .await
        .unwrap();

    let shared_state = AppState { db: pool };

    let app = collect_routes().with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
