use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::SystemTime,
};

use axum::{extract::FromRequestParts, http::request::Parts, Extension};
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, Jwk, JwkSet},
    DecodingKey, Validation,
};
use whynot_errors::{AppError, AppResult, SetupError, SetupResult};

fn lock_err<T>(s: impl ToString) -> AppResult<T> {
    Err(AppError::new(s))
}

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

        let Some(token) = token_parts.get(1) else {
            return lock_err("Invalid Token");
        };

        let header = decode_header(token).map_err(AppError::new)?;

        let mutex = auth_data.key_map.clone();

        let target_kid = header.kid.ok_or_else(|| AppError::new("Missing kid"))?;

        // If the mutex is poisoned, this will all burn anyway so it's fine to save an API call.
        let mut has_kid = true;

        // So the next few statements are weird. Need scoped if statements to remove the RAII
        // guards so the upgrade to write doesn't deadlock or be super annoying.

        if let Ok(key_sets) = mutex.read() {
            // km exists
            has_kid = key_sets.contains_key(&target_kid);
        }

        if !has_kid {
            // Load keyset
            let mut elapsed: u64 = 0;

            if let Ok(timer) = auth_data.timer.read() {
                if let Ok(actual_elapsed) = timer.elapsed() {
                    elapsed = actual_elapsed.as_secs();
                }
            }

            if elapsed > 60 {
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

                let Ok(mut timer) = auth_data.timer.write() else {
                    return lock_err("Failed to acquire write lock for timer");
                };

                *timer = SystemTime::now();
            }
        }

        let Ok(key_sets) = mutex.read() else {
            return lock_err("Failed to acquire second read lock for keys");
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
            validation.set_issuer(&[auth_data.options.issuer]);
            validation
        };

        let decoded_token =
            decode::<HashMap<String, serde_json::Value>>(token, &decoding_key, &validation)
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

pub fn locker<T>(obj: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(obj))
}

pub type Locker<T> = Arc<RwLock<T>>;

#[derive(Clone, Debug)]
pub struct AuthData {
    pub key_map: Locker<HashMap<String, Jwk>>,
    pub options: AuthOptions,
    pub timer: Locker<SystemTime>,
}

#[derive(Clone, Debug)]
pub struct AuthOptions {
    pub audience: String,
    pub issuer: String,
    pub origin: Option<String>,
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
