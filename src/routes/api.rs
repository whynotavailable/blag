use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{app_state::AppState, auth::AuthLayer, errors::server_error, models::SimpleResponse};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header::USER_AGENT, request::Parts, HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use tower_http::cors::CorsLayer;
use tracing::info;
use whynot_errors::{json_ok, JsonResult};

/// Simple endpoint to check the db connection is working.
/// TODO: Remove later to UI routes.
async fn db_healthcheck(
    State(state): State<AppState>,
    ExtractUserAgent(agent): ExtractUserAgent,
) -> JsonResult<SimpleResponse> {
    let result: (i32,) = sqlx::query_as("SELECT 12;")
        .fetch_one(&state.db)
        .await
        .map_err(server_error)?;

    json_ok(SimpleResponse::new(result.0))
}

struct ExtractUserAgent(HeaderValue);

impl<S> FromRequestParts<S> for ExtractUserAgent
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let Extension(auth_data) = parts
            .extract::<Extension<AuthData>>()
            .await
            .map_err(|err| err.into_response())?;

        let mutex = auth_data.key_map.clone();

        let mut has_kid = false;
        let test_key = "key".to_string();

        if let Ok(km) = mutex.read() {
            // km exists
            has_kid = km.contains_key(&test_key);
        }

        if !has_kid {
            // Load key
            let kmat = "simulated key data".to_string();

            if let Ok(mut km) = mutex.write() {
                km.insert(test_key.clone(), kmat);
            }
        }

        if let Ok(km) = mutex.read() {
            if let Some(key) = km.get(&test_key) {
                info!("{}", key);
            }
        }

        if let Some(user_agent) = parts.headers.get(USER_AGENT) {
            Ok(ExtractUserAgent(user_agent.clone()))
        } else {
            Err((StatusCode::BAD_REQUEST, "`User-Agent` header is missing").into_response())
        }
    }
}

#[derive(Clone, Debug)]
struct AuthData {
    key_map: Arc<RwLock<HashMap<String, String>>>,
}

pub fn api_routes() -> Router<AppState> {
    let auth_data = AuthData {
        key_map: Arc::new(RwLock::new(HashMap::new())),
    };
    Router::new()
        .route("/db-healthcheck", get(db_healthcheck))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
        .layer(Extension(auth_data))
}
