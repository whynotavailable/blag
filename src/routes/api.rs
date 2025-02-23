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
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, Jwk, JwkSet},
    DecodingKey, Validation,
};
use tower_http::cors::CorsLayer;
use tracing::info;
use whynot_errors::{json_ok, AppError, JsonResult};

/// Simple endpoint to check the db connection is working.
/// TODO: Remove later to UI routes.
async fn db_healthcheck(
    State(state): State<AppState>,
    Auth(agent): Auth,
) -> JsonResult<SimpleResponse> {
    let result: (i32,) = sqlx::query_as("SELECT 12;")
        .fetch_one(&state.db)
        .await
        .map_err(server_error)?;

    json_ok(SimpleResponse::new(result.0))
}

struct Auth(String);

impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let Extension(auth_data) = parts
            .extract::<Extension<AuthData>>()
            .await
            .map_err(AppError::new)?;

        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or(AppError::new("Missing Auth Header"))?
            .to_str()
            .map_err(AppError::new)?;

        let token_parts: Vec<&str> = auth_header.split(' ').collect();

        if token_parts.len() != 2 {
            return Err(AppError::new("Invalid Token"));
        }

        let header = decode_header(token_parts[1]).map_err(AppError::new)?;

        let mutex = auth_data.key_map.clone();

        let target_kid = header.kid.ok_or(AppError::new("Missing kid"))?;

        let mut has_kid = false;

        if let Ok(km) = mutex.read() {
            // km exists
            has_kid = km.contains_key(&target_kid);
        }

        if !has_kid {
            // Load keyset
            let kmat: JwkSet = serde_json::from_str("").map_err(AppError::new)?;

            if let Ok(mut km) = mutex.write() {
                for k in kmat.keys {
                    // The extra clone sucks, but it's only the once here for the key.
                    let kid = k
                        .common
                        .key_id
                        .clone()
                        .ok_or(AppError::new("keyset missing kid"))?;
                    km.entry(kid).or_insert(k);
                }
            }
        }

        let Ok(km_guard) = mutex.read() else {
            return Err(AppError::new("lock failed"));
        };

        let Some(key_material) = km_guard.get(&target_kid) else {
            return Err(AppError::new("no key"));
        };

        let decoding_key = match &key_material.algorithm {
            AlgorithmParameters::RSA(rsa) => {
                DecodingKey::from_rsa_components(&rsa.n, &rsa.e).map_err(AppError::new)?
            }
            _ => unreachable!("algorithm should be a RSA in this example"),
        };

        let validation = {
            let mut validation = Validation::new(header.alg);
            validation.set_audience(&[auth_data.audience]);
            validation
        };

        let decoded_token = decode::<HashMap<String, serde_json::Value>>(
            token_parts[1],
            &decoding_key,
            &validation,
        )
        .map_err(AppError::new)?;

        Ok(Auth(
            decoded_token
                .claims
                .get("sub")
                .ok_or(AppError::new("Missing sub"))?
                .to_string(),
        ))
    }
}

#[derive(Clone, Debug)]
struct AuthData {
    key_map: Arc<RwLock<HashMap<String, Jwk>>>,
    audience: String,
}

pub fn api_routes(audience: String) -> Router<AppState> {
    let auth_data = AuthData {
        key_map: Arc::new(RwLock::new(HashMap::new())),
        audience,
    };
    Router::new()
        .route("/db-healthcheck", get(db_healthcheck))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
        .layer(Extension(auth_data))
}
