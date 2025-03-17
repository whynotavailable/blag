use axum::http::StatusCode;
use tracing::info;
use whynot_errors::AppError;

pub fn not_found(obj: impl ToString) -> AppError {
    info!(code = 404, msg = obj.to_string(), "Custom Error");

    AppError {
        message: String::from("Not Found"),
        code: StatusCode::NOT_FOUND,
    }
}

pub fn server_error(obj: impl ToString) -> AppError {
    let mut err = AppError::new(obj);
    err.message = String::from("Server Error");
    err
}
