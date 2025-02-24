use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::SystemTime,
};

use crate::{
    app_state::AppState,
    auth::{Auth, AuthData, AuthOptions},
    errors::server_error,
    models::SimpleResponse,
};
use axum::{extract::State, routing::get, Extension, Router};
use tower_http::cors::CorsLayer;
use whynot_errors::{json_ok, JsonResult};

/// Simple endpoint to check the db connection is working.
/// TODO: Remove later to UI routes.
async fn db_healthcheck(
    State(state): State<AppState>,
    Auth(_auth): Auth,
) -> JsonResult<SimpleResponse> {
    let result: (i32,) = sqlx::query_as("SELECT 12;")
        .fetch_one(&state.db)
        .await
        .map_err(server_error)?;

    json_ok(SimpleResponse::new(result.0))
}

pub fn api_routes(auth_options: AuthOptions) -> Router<AppState> {
    let auth_data = AuthData {
        key_map: Arc::new(RwLock::new(HashMap::new())),
        options: auth_options,
        timer: Arc::new(RwLock::new(SystemTime::UNIX_EPOCH)),
    };
    Router::new()
        .route("/db-healthcheck", get(db_healthcheck))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
        .layer(Extension(auth_data))
}
