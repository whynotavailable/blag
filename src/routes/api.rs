use std::{collections::HashMap, time::SystemTime};

use crate::{
    app_state::AppState,
    auth::{locker, Auth, AuthData, AuthOptions},
    models::SimpleResponse,
    routes::pages::*,
};
use axum::{
    http::HeaderValue,
    http::Method,
    routing::{get, post},
    Extension, Router,
};

use tower_http::cors::CorsLayer;
use tracing::warn;
use whynot_errors::{AppError, JsonResult};

fn err<T>(obj: impl ToString) -> JsonResult<T> {
    Err(AppError::new(obj))
}

async fn noop(Auth(_sub): Auth) -> JsonResult<SimpleResponse> {
    err("Not Implemented")
}

pub fn api_routes(auth_options: AuthOptions) -> Router<AppState> {
    let original_origin = auth_options.origin.clone();

    let auth_data = AuthData {
        options: auth_options,
        key_map: locker(HashMap::new()),
        timer: locker(SystemTime::UNIX_EPOCH),
    };

    // The API will be made of RPCs so only GET and POST are needed.
    let cors_all: CorsLayer = CorsLayer::permissive().allow_methods([Method::GET, Method::POST]);

    // If an origin is provided, attempt to parse it and add it.
    let cors = match original_origin {
        Some(origin) => {
            if let Ok(o) = origin.parse::<HeaderValue>() {
                cors_all.allow_origin(o)
            } else {
                warn!("Origin [{}] failed to parse", origin);
                cors_all
            }
        }
        None => cors_all,
    };

    Router::new()
        // Post
        .route("/post_list", get(noop))
        .route("/post_get/{slug}", get(noop))
        .route("/post_update", post(noop))
        .route("/post_delete", post(noop))
        // Page
        .route("/page_list", get(page_list))
        .route("/page_get/{slug}", get(page_get))
        .route("/page_update/{slug}", post(page_update))
        .route("/page_delete", post(noop))
        // Layers
        .layer(cors)
        .layer(Extension(auth_data))
}
