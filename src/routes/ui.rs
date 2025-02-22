use crate::{
    app_state::AppState,
    errors::{not_found, server_error},
    models::SimpleResponse,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use tower_http::cors::CorsLayer;
use whynot_errors::{html_ok, HtmlResult};

#[debug_handler]
async fn get_search(State(state): State<AppState>) -> HtmlResult {
    let t = SimpleResponse::new("hi");

    state.refresh_if_needed().await?;

    let registry_lock = state.registry.clone();
    let registry = registry_lock.read().map_err(server_error)?;

    let contents = registry.render("list", &t).map_err(not_found)?;

    html_ok(contents)
}

#[allow(unused_variables)]
async fn get_page(State(state): State<AppState>, Path(slug): Path<String>) -> HtmlResult {
    state.refresh_if_needed().await?;
    html_ok("hi")
}

#[allow(unused_variables)]
async fn get_post(State(state): State<AppState>, Path(slug): Path<String>) -> HtmlResult {
    html_ok("")
}

pub fn ui_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_search))
        .route("/page/{slug}", get(get_page))
        .route("/post/{slug}", get(get_post))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
}
