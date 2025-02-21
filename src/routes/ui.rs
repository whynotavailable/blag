use crate::{
    app_state::AppState,
    errors::{not_found, server_error},
    models::SimpleResponse,
};
use axum::{extract::State, routing::get, Router};
use tower_http::cors::CorsLayer;
use whynot_errors::{html_ok, HtmlResult};

async fn get_search(State(state): State<AppState>) -> HtmlResult {
    state.refresh_if_needed().await?;

    let registry_lock = state.registry.clone();
    let registry = registry_lock.read().map_err(server_error)?;

    let t = SimpleResponse::new("hi");
    let contents = registry.render("list", &t).map_err(not_found)?;

    html_ok(contents)
}

pub fn ui_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_search))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
}
