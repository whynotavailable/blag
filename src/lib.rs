use std::path::Path;

use crate::routes::collect_routes;
use app_state::AppState;
use auth::AuthOptions;
use config::{Config, FileFormat};
use handlebars::{DirectorySourceOptions, Handlebars};
use sqlx::postgres::PgPoolOptions;
use tower_http::services::ServeDir;
use whynot_errors::{SetupError, SetupResult};

mod app_state;
mod auth;
mod errors;
pub mod models;
mod routes;

pub async fn setup(root_path: String) -> SetupResult {
    let root_path = Path::new(root_path.as_str());

    let settings = Config::builder()
        .add_source(
            config::File::new(
                root_path
                    .join("env.toml")
                    .to_str()
                    .ok_or_else(|| SetupError::new("no idea"))?,
                FileFormat::Toml,
            )
            .required(false),
        )
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .map_err(SetupError::new)?;

    let conn = settings.get_string("db").map_err(SetupError::new)?;
    let conn_cstr = conn.as_str();

    let include_api = settings.get_bool("include_api").unwrap_or(false);

    let mut registry = Handlebars::new();

    registry
        .register_templates_directory(
            root_path.join("templates"),
            DirectorySourceOptions::default(),
        )
        .map_err(SetupError::new)?;

    let db = PgPoolOptions::new()
        .max_connections(100)
        .connect(conn_cstr)
        .await
        .map_err(SetupError::new)?;

    let shared_state = AppState { db, registry };

    let audience = settings.get_string("audience").unwrap_or(String::from(""));
    let issuer = settings.get_string("issuer").unwrap_or(String::from(""));
    let origin = settings.get_string("origin").ok();

    let auth_options = AuthOptions {
        audience,
        issuer,
        origin,
    };

    if include_api {
        auth_options.validate()?;
    }

    let app = collect_routes(include_api, auth_options)
        .with_state(shared_state)
        .nest_service("/assets", ServeDir::new(root_path.join("assets")));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030")
        .await
        .map_err(SetupError::new)?;

    println!("starting up");
    axum::serve(listener, app).await.map_err(SetupError::new)?;

    Ok(())
}
