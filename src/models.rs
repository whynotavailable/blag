use serde::Serialize;
use sqlx::PgPool;
use whynot_errors::{json_ok, JsonResult};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: PgPool,
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

#[derive(Serialize, Debug)]
pub struct DataSource {
    pub id: sqlx::types::Uuid,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        let cst: &str = "hi";
        assert_eq!(SimpleResponse::new(cst).value, "hi");

        let s: String = format!("{} {}", "hi", "dave");
        assert_eq!(SimpleResponse::new(s).value, "hi dave");
    }
}
