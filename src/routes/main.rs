use crate::{app_state::AppState, models::SimpleResponse};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use whynot_errors::{json_ok, AppError, JsonResult};

#[derive(Deserialize, Debug)]
pub struct HandoffRequest {
    pub app_id: Uuid,
    pub env: String,
    pub artifact: String,
}

async fn handoff(Json(body): Json<HandoffRequest>) -> JsonResult<SimpleResponse> {
    SimpleResponse::json(format!("{:?}", body))
}

async fn db_healthcheck(State(state): State<AppState>) -> JsonResult<SimpleResponse> {
    let result: (i32,) = sqlx::query_as("SELECT 12;")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::from)?;

    json_ok(SimpleResponse::new(result.0))
}

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/handoff", post(handoff))
        .route("/db-healthcheck", get(db_healthcheck))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
}
