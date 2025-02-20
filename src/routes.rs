use crate::{app_state::AppState, routes::main::main_routes};
use axum::{handler::HandlerWithoutStateExt, Router};
use whynot_errors::AppError;

mod main;
mod ui;

pub fn collect_routes() -> Router<AppState> {
    async fn handle_404() -> AppError {
        AppError::not_found()
    }

    let service = handle_404.into_service();

    let api_routes = Router::new().merge(main_routes());

    Router::new().nest("/api", api_routes.fallback_service(service))
}
