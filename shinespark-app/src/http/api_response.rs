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
        let status_code = match value {
            shinespark::Error::Internal(_)
            | shinespark::Error::DatabaseError(_)
            | shinespark::Error::IllegalState(_)
            | shinespark::Error::NotImplemented => StatusCode::INTERNAL_SERVER_ERROR,
            shinespark::Error::NotFound
            | shinespark::Error::AlreadyExists
            | shinespark::Error::InvalidCredentials => StatusCode::BAD_REQUEST,
            shinespark::Error::UnAuthorized => StatusCode::UNAUTHORIZED,
        };
        Self {
            status_code,
            code: value.code(),
            message: value.to_string(),
        }
    }
}
