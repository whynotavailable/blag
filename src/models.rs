use std::{fmt::Display, str};

use handlebars::Handlebars;
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use whynot_errors::{json_ok, JsonResult};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: PgPool,
    // There seemingly isn't a nonref template library that supports async updates. Will have to
    // make my own eventually.
    pub registry: Handlebars<'static>,
}

#[derive(Serialize)]
pub struct SimpleResponse {
    pub value: String,
}

impl SimpleResponse {
    pub fn new(value: impl ToString) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn json(value: impl ToString) -> JsonResult<Self> {
        json_ok(SimpleResponse::new(value))
    }
}

#[derive(FromRow)]
pub struct TemplateData {
    pub key: String,
    pub template: String,
}

// TODO: Move this to the errors lib.
#[derive(Debug)]
pub struct SetupError {
    pub msg: String,
}

impl Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Setup Error: {}", self.msg)
    }
}

impl SetupError {
    pub fn new(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

pub type SetupResult = Result<(), SetupError>;
