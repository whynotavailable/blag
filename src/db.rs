use std::fmt::Display;

use sqlx::{query, query_as, PgPool};
use whynot_errors::{AppError, AppResult};

use crate::errors;

#[derive(Debug)]
pub enum ConfigKeys {
    TemplateNonce,
}

impl Display for ConfigKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub async fn get_config(pool: &PgPool, key: ConfigKeys) -> AppResult<String> {
    let row: (String,) = query_as("SELECT value from config WHERE key = $1;")
        .bind(key.to_string())
        .fetch_one(pool)
        .await
        .map_err(errors::not_found)?;

    Ok(row.0)
}

pub async fn set_config(pool: &PgPool, key: ConfigKeys, value: String) -> AppResult<()> {
    query("CALL set_config($1, $2);")
        .bind(key.to_string())
        .bind(value)
        .execute(pool)
        .await
        .map_err(AppError::new)?;

    Ok(())
}
