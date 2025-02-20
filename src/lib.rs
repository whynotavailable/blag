use std::{
    sync::{Arc, RwLock},
    time::SystemTime,
};

use crate::routes::collect_routes;
use app_state::AppState;
use handlebars::Handlebars;
use models::{SetupError, SetupResult};
use sqlx::postgres::PgPoolOptions;

mod app_state;
pub mod models;
mod routes;

pub async fn setup() -> SetupResult {
    // This will become mutable later on lol. I didn't know that was possible
    let registry = Handlebars::new();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect("postgres://postgres:garden@localhost/blag")
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

    let app = collect_routes().with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, app).await.map_err(SetupError::new)?;

    Ok(())
}
