use crate::{
    app_state::AppState,
    errors::{self, not_found, server_error},
    models::SimpleResponse,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use sqlx::query_as;
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

    let r: (String,) = query_as("SELECT content FROM pages WHERE slug = $1;")
        .bind(slug)
        .fetch_one(&state.db)
        .await
        .map_err(errors::not_found)?;

    html_ok(r.0)
}

#[allow(unused_variables)]
async fn get_post(State(state): State<AppState>, Path(slug): Path<String>) -> HtmlResult {
    html_ok("")
}

async fn lt_lock(State(state): State<AppState>) -> HtmlResult {
    state.refresh_if_needed().await?;
    html_ok("lt")
}

async fn lt_reload(State(state): State<AppState>) -> HtmlResult {
    html_ok(state.get_db_nonce().await)
}

pub fn ui_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_search))
        .route("/page/{slug}", get(get_page))
        .route("/post/{slug}", get(get_post))
        .route("/lt", get(lt_lock))
        .route("/reload", get(lt_reload))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
}
