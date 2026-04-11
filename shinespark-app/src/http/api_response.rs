use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ApiResponse<T> {
    pub code: &'static str,
    pub data: T,
}

#[derive(Debug)]
pub struct ApiError {
    pub status_code: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { code: "Ok", data }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let body = axum::response::Json(self);
        (StatusCode::OK, body).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = axum::response::Json(serde_json::json!({
            "code": self.code,
            "message": self.message,
        }));
        (self.status_code, body).into_response()
    }
}

pub type ApiResult<T> = Result<ApiResponse<T>, ApiError>;

impl From<shinespark::Error> for ApiError {
    fn from(value: shinespark::Error) -> Self {
        match value {
            shinespark::Error::Internal(_) => Self {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                code: "INTERNAL",
                message: value.to_string(),
            },
            shinespark::Error::IllegalState(_) => Self {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                code: "ILLEGAL_STATE",
                message: value.to_string(),
            },
            shinespark::Error::NotImplemented => Self {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                code: "NOT_IMPLEMENTED",
                message: value.to_string(),
            },
            shinespark::Error::UnAuthorized => Self {
                status_code: StatusCode::UNAUTHORIZED,
                code: "UN_AUTHORIZED",
                message: value.to_string(),
            },
            shinespark::Error::DatabaseError(_) => Self {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                code: "DATABASE_ERROR",
                message: value.to_string(),
            },
            shinespark::Error::NotFound => Self {
                status_code: StatusCode::NOT_FOUND,
                code: "NOT_FOUND",
                message: value.to_string(),
            },
            shinespark::Error::AlreadyExists => Self {
                status_code: StatusCode::BAD_REQUEST,
                code: "ALREADY_EXISTS",
                message: value.to_string(),
            },
            shinespark::Error::InvalidCredentials => Self {
                status_code: StatusCode::BAD_REQUEST,
                code: "INVALID_CREDENTIALS",
                message: value.to_string(),
            },
        }
    }
}
