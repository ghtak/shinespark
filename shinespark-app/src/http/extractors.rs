use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use shinespark_identity::entities::UserAggregate;
use tower_sessions::Session;

use crate::http::ApiError;

pub const USER_SESSION_KEY: &str = "user_session";

#[derive(Debug)]
pub struct CurrentUser(pub UserAggregate);

impl<S> OptionalFromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let session = parts.extensions.get::<Session>().ok_or(ApiError::from(
            shinespark::Error::IllegalState(std::borrow::Cow::Borrowed("session not found")),
        ))?;
        let user = session.get::<UserAggregate>(USER_SESSION_KEY).await.map_err(|e| {
            ApiError::from(shinespark::Error::Internal(
                anyhow::anyhow!(e).context("get user from session failed"),
            ))
        })?;
        Ok(user.map(CurrentUser))
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        <CurrentUser as OptionalFromRequestParts<S>>::from_request_parts(parts, state)
            .await?
            .ok_or(ApiError::from(shinespark::Error::UnAuthorized))
    }
}
