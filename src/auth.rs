use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use axum::{extract::FromRequestParts, http::request::Parts, Extension};
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, Jwk, JwkSet},
    DecodingKey, Validation,
};
use whynot_errors::{AppError, SetupError, SetupResult};

pub struct Auth(pub String);

impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let Extension(auth_data) = parts
            .extract::<Extension<AuthData>>()
            .await
            .map_err(AppError::new)?;

        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or_else(|| AppError::new("Missing Auth Header"))?
            .to_str()
            .map_err(AppError::new)?;

        let token_parts: Vec<&str> = auth_header.split(' ').collect();

        if token_parts.len() != 2 {
            return Err(AppError::new("Invalid Token"));
        }

        let header = decode_header(token_parts[1]).map_err(AppError::new)?;

        let mutex = auth_data.key_map.clone();

        let target_kid = header.kid.ok_or_else(|| AppError::new("Missing kid"))?;

        let mut has_kid = false;

        // So the next few statements are weird. Need scoped if statements to remove the RAII
        // guards so the upgrade to write doesn't deadlock or be super annoying.

        if let Ok(key_sets) = mutex.read() {
            // km exists
            has_kid = key_sets.contains_key(&target_kid);
        }

        if !has_kid {
            // Load keyset
            let sets: JwkSet =
                reqwest::get(format!("{}.well-known/jwks.json", auth_data.options.issuer))
                    .await
                    .map_err(AppError::new)?
                    .json()
                    .await
                    .map_err(AppError::new)?;

            if let Ok(mut key_sets) = mutex.write() {
                for key in sets.keys {
                    // The extra clone sucks, but it's only the once here for the key.
                    let kid = key
                        .common
                        .key_id
                        .clone()
                        .ok_or_else(|| AppError::new("keyset missing kid"))?;
                    key_sets.entry(kid).or_insert(key);
                }
            }
        }

        let Ok(key_sets) = mutex.read() else {
            return Err(AppError::new("lock failed"));
        };

        let Some(key_material) = key_sets.get(&target_kid) else {
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
            validation.set_audience(&[auth_data.options.audience]);
            // TODO: move this to auth config thing.
            validation.set_issuer(&[auth_data.options.issuer]);
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
                .ok_or_else(|| AppError::new("Missing sub"))?
                .to_string(),
        ))
    }
}

#[derive(Clone, Debug)]
pub struct AuthData {
    pub key_map: Arc<RwLock<HashMap<String, Jwk>>>,
    pub options: AuthOptions,
}

#[derive(Clone, Debug)]
pub struct AuthOptions {
    pub audience: String,
    pub issuer: String,
}

impl AuthOptions {
    pub fn validate(&self) -> SetupResult {
        if self.audience.is_empty() {
            return Err(SetupError::new("Missing audience"));
        } else if self.issuer.is_empty() {
            return Err(SetupError::new("Missing issuer"));
        }

        Ok(())
    }
}
