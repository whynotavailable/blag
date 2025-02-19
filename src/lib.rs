use crate::models::AppState;
use crate::routes::collect_routes;
use handlebars::Handlebars;
use models::{SetupError, SetupResult, TemplateData};
use sqlx::postgres::PgPoolOptions;

pub mod models;
mod routes;

pub async fn setup() -> SetupResult {
    #[allow(unused_mut)]
    let mut registry = Handlebars::new();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect("postgres://postgres:garden@localhost/handoff")
        .await
        .map_err(SetupError::new)?;

    let templates: Vec<TemplateData> = sqlx::query_as("SELECT * FROM templates;")
        .fetch_all(&db)
        .await
        .map_err(SetupError::new)?;

    for template in templates {
        registry
            .register_template_string(template.key.as_str(), template.template.as_str())
            .map_err(SetupError::new)?;
    }

    let shared_state = AppState { db, registry };

    let app = collect_routes().with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, app).await.map_err(SetupError::new)?;

    Ok(())
}
