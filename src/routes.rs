use crate::models::AppState;
use crate::routes::main::main_routes;
use axum::{handler::HandlerWithoutStateExt, Router};
use machine::machine_routes;
use tower_http::services::{ServeDir, ServeFile};
use whynot_errors::AppError;

mod machine;
mod main;

pub fn collect_routes() -> Router<AppState> {
    async fn handle_404() -> AppError {
        AppError::not_found()
    }

    let service = handle_404.into_service();

    let serve_dir = ServeDir::new("public").not_found_service(ServeFile::new("public/index.html"));

    let api_routes = Router::new().merge(main_routes()).merge(machine_routes());

    Router::new()
        .fallback_service(serve_dir)
        .nest("/api", api_routes.fallback_service(service))
}
