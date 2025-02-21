use crate::{app_state::AppState, errors::CustomErrors, models::SimpleResponse};
use axum::{extract::State, routing::get, Router};
use tower_http::cors::CorsLayer;
use whynot_errors::{html_ok, AppError, HtmlResult};

async fn get_search(State(state): State<AppState>) -> HtmlResult {
    state.refresh_if_needed().await?;

    let registry_lock = state.registry.clone();
    let registry = registry_lock.read().map_err(AppError::new)?;

    let t = SimpleResponse::new("hi");
    let contents = registry.render("list", &t).map_err(AppError::not_found)?;

    html_ok(contents)
}

pub fn ui_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_search))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
}
