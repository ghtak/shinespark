use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use shinespark::config::JwtConfig;
use shinespark_identity::infra::{JwtClaims, JwtTokenPair};
use time::Duration;

use crate::{AppContainer, http::ApiError};

enum TokenStatus {
    Valid(JwtClaims),
    TryRefresh,
    Invalid,
}

fn check_access_token(container: &AppContainer, jar: &CookieJar) -> TokenStatus {
    let Some(cookie) = jar.get("access_token") else {
        return TokenStatus::TryRefresh;
    };
    let token = cookie.value();
    match container.jwt_service.verify(token) {
        Ok(claims) if claims.token_type == "access" => TokenStatus::Valid(claims),
        Err(_) if container.jwt_service.is_expired(token) => TokenStatus::TryRefresh,
        _ => TokenStatus::Invalid,
    }
}

async fn try_refresh(
    container: &AppContainer,
    jar: &CookieJar,
) -> Option<(JwtClaims, JwtTokenPair)> {
    let token = jar.get("refresh_token")?.value().to_owned();
    let pair =
        container.jwt_ident_usecase.refresh(&mut container.db.handle(), &token).await.ok()?;
    let claims = container.jwt_service.verify(&pair.access_token).ok()?;
    Some((claims, pair))
}

pub async fn auth_middleware(
    State(container): State<Arc<AppContainer>>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Response {
    match check_access_token(&container, &jar) {
        TokenStatus::Valid(claims) => {
            req.extensions_mut().insert(claims);
            return next.run(req).await;
        }
        TokenStatus::Invalid => return Redirect::to("/auth/login").into_response(),
        TokenStatus::TryRefresh => {}
    }

    match try_refresh(&container, &jar).await {
        Some((claims, pair)) => {
            req.extensions_mut().insert(claims);
            let res = next.run(req).await;
            (CookieJarJwt::build(&pair, &container.config.jwt), res).into_response()
        }
        None => Redirect::to("/auth/login").into_response(),
    }
}

#[allow(dead_code)]
pub struct CookieJwtUser(pub JwtClaims);

impl<S: Send + Sync> FromRequestParts<S> for CookieJwtUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<JwtClaims>()
            .cloned()
            .map(CookieJwtUser)
            .ok_or_else(|| ApiError::from(shinespark::Error::UnAuthorized))
    }
}

pub struct CookieJarJwt;

impl CookieJarJwt {
    pub fn build<'a>(pair: &'a JwtTokenPair, jwt_config: &'a JwtConfig) -> CookieJar {
        let build_cookie = |name: &'static str, value: String, ttl: i64| {
            Cookie::build((name, value))
                .http_only(true)
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::seconds(ttl))
                .secure(jwt_config.secure_cookie)
                .build()
        };
        CookieJar::new()
            .add(build_cookie(
                "access_token",
                pair.access_token.to_owned(),
                jwt_config.access_token_ttl_secs,
            ))
            .add(build_cookie(
                "refresh_token",
                pair.refresh_token.to_owned(),
                jwt_config.refresh_token_ttl_secs,
            ))
    }

    pub fn clear() -> CookieJar {
        CookieJar::new()
            .add(Cookie::build(("access_token", "")).path("/").max_age(Duration::ZERO).build())
            .add(Cookie::build(("refresh_token", "")).path("/").max_age(Duration::ZERO).build())
    }
}
