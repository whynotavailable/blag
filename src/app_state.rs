use std::fmt::Debug;

use handlebars::Handlebars;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: PgPool,
    pub registry: Handlebars<'static>,
}
