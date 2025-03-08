use std::{collections::HashMap, time::SystemTime};

use crate::{
    app_state::AppState,
    auth::{locker, Auth, AuthData, AuthOptions},
    errors,
    models::SimpleResponse,
};
use axum::{
    extract::{Path, State},
    http::{HeaderValue, Method},
    routing::{get, post},
    Extension, Json, Router,
};

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use whynot_errors::{json_ok, AppError, JsonResult};

fn err<T>(obj: impl ToString) -> JsonResult<T> {
    Err(AppError::new(obj))
}

async fn noop(Auth(_sub): Auth) -> JsonResult<SimpleResponse> {
    err("Not Implemented")
}

#[derive(FromRow, Serialize, Debug)]
struct PageListItem {
    slug: String,
    title: String,
}

async fn page_list(
    State(state): State<AppState>,
    Auth(_sub): Auth,
) -> JsonResult<Vec<PageListItem>> {
    let sql = "SELECT slug, title FROM pages;";

    json_ok(
        query_as(sql)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::new)?,
    )
}

// Will be used for both sides.
#[derive(FromRow, Serialize, Deserialize, Debug)]
struct PageEdit {
    title: String,
    raw: String,
}

async fn page_get(
    State(state): State<AppState>,
    Auth(_sub): Auth,
    Path(slug): Path<String>,
) -> JsonResult<PageEdit> {
    let sql = "SELECT title, raw FROM pages WHERE slug = $1;";

    json_ok(
        query_as(sql)
            .bind(slug)
            .fetch_one(&state.db)
            .await
            .map_err(errors::not_found)?,
    )
}

async fn page_update(
    State(state): State<AppState>,
    Auth(_sub): Auth,
    Path(slug): Path<String>,
    Json(body): Json<PageEdit>,
) -> JsonResult<SimpleResponse> {
    let sql = "UPDATE pages SET title = $1, raw = $2, content = $3 WHERE slug = $4;";
    let html = markdown::to_html(&body.raw);

    query(sql)
        .bind(body.title)
        .bind(body.raw)
        .bind(html)
        .bind(slug)
        .execute(&state.db)
        .await
        .map_err(AppError::new)?;

    SimpleResponse::json("ok")
}

pub fn api_routes(auth_options: AuthOptions) -> Router<AppState> {
    let original_origin = auth_options.origin.clone();

    let auth_data = AuthData {
        options: auth_options,
        key_map: locker(HashMap::new()),
        timer: locker(SystemTime::UNIX_EPOCH),
    };

    // The API will be made of RPCs so only GET and POST are needed.
    let cors_all: CorsLayer = CorsLayer::permissive(); //.allow_methods([Method::GET, Method::POST]);

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
