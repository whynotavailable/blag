use crate::{app_state::AppState, auth::AuthOptions, routes::api::api_routes};
use axum::Router;
use ui::ui_routes;

mod api;
mod ui;

mod pages;

pub fn collect_routes(include_api: bool, auth_options: AuthOptions) -> Router<AppState> {
    let router = Router::new().merge(ui_routes());

    if include_api {
        router.nest("/api", api_routes(auth_options))
    } else {
        router
    }
}
