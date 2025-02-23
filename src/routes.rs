use crate::{app_state::AppState, routes::api::api_routes};
use axum::Router;
use ui::ui_routes;

mod api;
mod ui;

pub fn collect_routes(include_api: bool, audience: String) -> Router<AppState> {
    let router = Router::new().merge(ui_routes());

    if include_api {
        router.nest("/api", api_routes(audience))
    } else {
        router
    }
}
