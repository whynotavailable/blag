use crate::models::{AppState, SimpleResponse};
use axum::{routing::get, Router};
use whynot_errors::{json_ok, JsonResult};

async fn status() -> JsonResult<SimpleResponse> {
    json_ok(SimpleResponse::new("Ok"))
}

pub fn machine_routes() -> Router<AppState> {
    Router::new().route("/status", get(status))
}
