use crate::{app_state::AppState, errors::server_error, models::SimpleResponse};
use axum::{extract::State, routing::get, Router};
use tower_http::cors::CorsLayer;
use whynot_errors::{json_ok, JsonResult};

/// Simple endpoint to check the db connection is working.
/// TODO: Remove later to UI routes.
async fn db_healthcheck(State(state): State<AppState>) -> JsonResult<SimpleResponse> {
    let result: (i32,) = sqlx::query_as("SELECT 12;")
        .fetch_one(&state.db)
        .await
        .map_err(server_error)?;

    json_ok(SimpleResponse::new(result.0))
}

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/db-healthcheck", get(db_healthcheck))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
}
