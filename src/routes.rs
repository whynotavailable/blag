use crate::{app_state::AppState, routes::main::api_routes};
use axum::Router;
use ui::ui_routes;

mod main;
mod ui;

pub fn collect_routes() -> Router<AppState> {
    Router::new().merge(ui_routes()).nest("/api", api_routes())
}
