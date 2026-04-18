use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use shinespark_identity::infra::JwtClaims;

use crate::AppContainer;
use crate::http::ApiError;

pub struct JwtUser(pub JwtClaims);

impl<S> FromRequestParts<S> for JwtUser
where
    S: Send + Sync,
    Arc<AppContainer>: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let container = Arc::<AppContainer>::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::from(shinespark::Error::UnAuthorized))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::from(shinespark::Error::UnAuthorized))?;

        let claims = container
            .jwt_service
            .verify(token)
            .map_err(|_| ApiError::from(shinespark::Error::UnAuthorized))?;

        Ok(JwtUser(claims))
    }
}
