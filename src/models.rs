use std::fmt::Display;

use serde::Serialize;
use sqlx::FromRow;
use whynot_errors::{json_ok, JsonResult};

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
