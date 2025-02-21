use std::{
    sync::{Arc, RwLock},
    time::SystemTime,
};

use crate::routes::collect_routes;
use app_state::AppState;
use config::Config;
use errors::CustomErrors;
use handlebars::Handlebars;
use sqlx::postgres::PgPoolOptions;
use whynot_errors::{AppError, SetupError, SetupResult};

mod app_state;
mod errors;
pub mod models;
mod routes;

pub async fn setup() -> SetupResult {
    let _ = AppError::not_found("lol");

    let settings = Config::builder()
        .add_source(config::File::with_name("env"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .map_err(SetupError::new)?;

    let conn = settings.get_string("db").map_err(SetupError::new)?;
    let conn_cstr = conn.as_str();

    let include_api = settings.get_bool("include_api").unwrap_or(false);

    // This will become mutable later on lol. I didn't know that was possible
    let registry = Handlebars::new();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(conn_cstr)
        .await
        .map_err(SetupError::new)?;

    let shared_state = AppState {
        db,
        registry: Arc::new(RwLock::new(registry)),
        timer: Arc::new(RwLock::new(SystemTime::now())),
    };

    shared_state
        .reload_templates()
        .await
        .map_err(SetupError::new)?;

    let app = collect_routes(include_api).with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, app).await.map_err(SetupError::new)?;

    Ok(())
}
