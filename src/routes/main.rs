use crate::{app_state::AppState, models::SimpleResponse};
use axum::{routing::post, Json, Router};
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use whynot_errors::JsonResult;

#[derive(Deserialize, Debug)]
pub struct HandoffRequest {
    pub app_id: Uuid,
    pub env: String,
    pub artifact: String,
}

async fn handoff(Json(body): Json<HandoffRequest>) -> JsonResult<SimpleResponse> {
    SimpleResponse::json(format!("{:?}", body))
}

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/handoff", post(handoff))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
}
