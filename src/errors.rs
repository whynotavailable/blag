use axum::http::StatusCode;
use tracing::info;
use whynot_errors::AppError;

pub trait CustomErrors {
    fn not_found(obj: impl ToString) -> AppError;
}

fn log(msg: String, code: StatusCode) {
    info!(code = code.as_u16(), msg = msg, "Custom Error");
}

impl CustomErrors for AppError {
    fn not_found(obj: impl ToString) -> AppError {
        log(obj.to_string(), StatusCode::NOT_FOUND);

        AppError {
            message: "Not Found".to_string(),
            code: StatusCode::NOT_FOUND,
        }
    }
}
