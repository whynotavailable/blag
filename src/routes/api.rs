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
use axum::{
    extract::State,
    http::{HeaderValue, Method},
    routing::get,
    Extension, Router,
};
use tower_http::cors::CorsLayer;
use tracing::warn;
use whynot_errors::{json_ok, JsonResult, SetupError};

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
    let original_origin = auth_options.origin.clone();

    let auth_data = AuthData {
        key_map: Arc::new(RwLock::new(HashMap::new())),
        options: auth_options,
        timer: Arc::new(RwLock::new(SystemTime::UNIX_EPOCH)),
    };

    // The API will be made of RPCs so only GET and POST are needed.
    let cors_all: CorsLayer = CorsLayer::permissive().allow_methods([Method::GET, Method::POST]);

    let cors = match original_origin {
        Some(origin) => {
            if let Ok(o) = origin.parse::<HeaderValue>() {
                cors_all.allow_origin(o)
            } else {
                warn!("Origin {} failed to parse", origin);
                cors_all
            }
        }
        None => cors_all,
    };

    Router::new()
        .route("/db-healthcheck", get(db_healthcheck))
        .layer(cors) // TODO: fix this lol
        .layer(Extension(auth_data))
}
