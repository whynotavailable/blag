use axum::http::StatusCode;
use tracing::{error, info};
use whynot_errors::AppError;

pub trait CustomErrors {
    fn not_found(obj: impl ToString) -> AppError;
    fn server_error(obj: impl ToString) -> AppError;
}

// These error handlers consume the input, and log the output.
// `not_found` also changes the severity.
impl CustomErrors for AppError {
    fn not_found(obj: impl ToString) -> AppError {
        info!(code = 404, msg = obj.to_string(), "Custom Error");

        AppError {
            message: "Not Found".to_string(),
            code: StatusCode::NOT_FOUND,
        }
    }

    fn server_error(obj: impl ToString) -> AppError {
        error!("{}", obj.to_string());

        AppError {
            message: "Server Error".to_string(),
            code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
