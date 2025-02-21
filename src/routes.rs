use crate::{app_state::AppState, routes::main::api_routes};
use axum::Router;
use ui::ui_routes;

mod main;
mod ui;

pub fn collect_routes(include_api: bool) -> Router<AppState> {
    let router = Router::new().merge(ui_routes());

    if include_api {
        router.nest("/api", api_routes())
    } else {
        router
    }
}
