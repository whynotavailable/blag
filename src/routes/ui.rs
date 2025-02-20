use crate::{app_state::AppState, models::SimpleResponse};
use axum::{extract::State, http::StatusCode, routing::get, Router};
use tower_http::cors::CorsLayer;
use whynot_errors::{html_ok, AppError, HtmlResult};

async fn get_search(State(state): State<AppState>) -> HtmlResult {
    let registry_lock = state.registry.clone();
    let registry = registry_lock.read().map_err(AppError::from)?;

    let t = SimpleResponse::new("hi");
    let contents = registry
        .render("list", &t)
        .map_err(AppError::code(StatusCode::NOT_FOUND))?;

    html_ok(contents)
}

pub fn ui_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_search))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
}
