use serde::Serialize;
use sqlx::FromRow;
use whynot_errors::json_ok;
use whynot_errors::JsonResult;

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
