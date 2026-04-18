pub mod identity {
    pub mod dto {

        #[derive(Debug, serde::Deserialize)]
        pub struct LoginRequest {
            pub email: String,
            pub password: String,
        }
    }

    use std::sync::Arc;

    use axum::Router;

    use crate::AppContainer;

    mod session {
        use std::sync::Arc;

        use axum::{Json, Router, extract::State};
        use shinespark_identity::{entities::UserAggregate, usecases::LoginCommand};

        use crate::{
            AppContainer,
            http::{
                ApiResponse, ApiResult,
                session::{CurrentUser, Session, USER_SESSION_KEY},
            },
        };

        async fn login(
            State(container): State<Arc<AppContainer>>,
            session: Session,
            Json(command): Json<super::dto::LoginRequest>,
        ) -> ApiResult<UserAggregate> {
            let user_aggregate = container
                .login_usecase
                .login(
                    &mut container.db.handle(),
                    LoginCommand::Local {
                        email: command.email,
                        password: command.password,
                    },
                )
                .await?;
            session
                .insert(USER_SESSION_KEY, user_aggregate.clone())
                .await
                .map_err(|e| shinespark::Error::Internal(anyhow::anyhow!(e).context("context")))?;
            Ok(ApiResponse::new(user_aggregate))
        }

        async fn logout(session: Session, _user: CurrentUser) -> ApiResult<()> {
            session
                .remove_value(USER_SESSION_KEY)
                .await
                .map_err(|e| shinespark::Error::Internal(anyhow::anyhow!(e).context("context")))?;
            Ok(ApiResponse::new(()))
        }

        async fn me(user: CurrentUser) -> ApiResult<UserAggregate> {
            Ok(ApiResponse::new(user.0))
        }

        pub fn routes() -> Router<Arc<AppContainer>> {
            Router::new()
                .route("/identity/session/login", axum::routing::post(login))
                .route("/identity/session/logout", axum::routing::post(logout))
                .route("/identity/session/me", axum::routing::get(me))
        }
    }

    mod jwt {
        use std::sync::Arc;

        use axum::{Json, Router, extract::State};
        use serde::{Deserialize, Serialize};
        use shinespark_identity::{infra::JwtClaims, usecases::LoginCommand};

        use crate::{
            AppContainer,
            http::{ApiResponse, ApiResult, jwt::JwtUser},
        };

        #[derive(Debug, Serialize)]
        pub struct JwtTokenResponse {
            pub access_token: String,
            pub refresh_token: String,
        }

        #[derive(Debug, Deserialize)]
        pub struct JwtRefreshRequest {
            pub refresh_token: String,
        }

        /// 로그인 처리
        ///
        /// 이메일과 비밀번호를 사용하여 로그인하고, access_token과 refresh_token을 발급합니다.
        async fn login(
            State(container): State<Arc<AppContainer>>,
            Json(command): Json<super::dto::LoginRequest>,
        ) -> ApiResult<JwtTokenResponse> {
            let pair = container
                .jwt_ident_usecase
                .login(
                    &mut container.db.handle(),
                    LoginCommand::Local {
                        email: command.email,
                        password: command.password,
                    },
                )
                .await?;
            Ok(ApiResponse::new(JwtTokenResponse {
                access_token: pair.access_token,
                refresh_token: pair.refresh_token,
            }))
        }

        /// 로그아웃 처리
        ///
        /// refresh_token을 무효화하여 현재 발급된 access_token이 더 이상 갱신되지 않도록 처리합니다.
        /// 주의: 기존에 발급된 access_token은 만료 시점까지 유효합니다.
        /// 즉시 접근을 차단하는 실시간(real-time) 로그아웃이 필요한 경우, 별도의 블랙리스트(blacklist)를 통해 관리해야 합니다.
        async fn logout(
            State(container): State<Arc<AppContainer>>,
            user: JwtUser,
        ) -> ApiResult<()> {
            container.jwt_ident_usecase.logout(&mut container.db.handle(), &user.0.sub).await?;
            Ok(ApiResponse::new(()))
        }

        /// 토큰 갱신 처리
        ///
        /// refresh_token을 사용하여 새로운 access_token과 refresh_token을 발급합니다.
        /// refresh_token이 유효하지 않거나 만료된 경우 갱신할 수 없습니다.
        async fn refresh_token(
            State(container): State<Arc<AppContainer>>,
            Json(body): Json<JwtRefreshRequest>,
        ) -> ApiResult<JwtTokenResponse> {
            let pair = container
                .jwt_ident_usecase
                .refresh(&mut container.db.handle(), &body.refresh_token)
                .await?;
            Ok(ApiResponse::new(JwtTokenResponse {
                access_token: pair.access_token,
                refresh_token: pair.refresh_token,
            }))
        }

        /// 현재 토큰 정보 조회
        ///
        /// access_token에 포함된 정보를 반환합니다.
        async fn me(user: JwtUser) -> ApiResult<JwtClaims> {
            Ok(ApiResponse::new(user.0))
        }

        pub fn routes() -> Router<Arc<AppContainer>> {
            Router::new()
                .route("/identity/jwt/login", axum::routing::post(login))
                .route("/identity/jwt/logout", axum::routing::post(logout))
                .route("/identity/jwt/refresh", axum::routing::post(refresh_token))
                .route("/identity/jwt/me", axum::routing::get(me))
        }
    }

    pub fn routes() -> Router<Arc<AppContainer>> {
        Router::new().merge(session::routes()).merge(jwt::routes())
    }
}
