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
        use tracing::info;

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
            info!("me: {}", user.0.sub);
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

    mod oauth2 {
        use std::sync::Arc;

        use axum::{
            Router,
            extract::{Path, Query, State},
            response::Redirect,
        };
        use serde::{Deserialize, Serialize};
        use shinespark_identity::{
            entities::AuthProvider,
            usecases::{LoginCommand, SocialCallbackCommand, SocialLoginCommand},
        };

        use crate::{
            AppContainer,
            http::{ApiResponse, ApiResult, api_response::ApiError},
        };

        #[derive(Debug, Serialize)]
        pub struct OAuthTokenResponse {
            pub access_token: String,
            pub refresh_token: String,
        }

        #[derive(Debug, Deserialize)]
        pub struct CallbackParams {
            pub code: String,
            pub state: String,
        }

        async fn login(
            State(container): State<Arc<AppContainer>>,
            Path(provider): Path<String>,
        ) -> Result<Redirect, ApiError> {
            let usecase = provider_usecase(&container, &provider)?;
            let state = uuid::Uuid::new_v4().to_string();
            let url = usecase.login(SocialLoginCommand { state }).await?;
            Ok(Redirect::temporary(&url))
        }

        async fn callback(
            State(container): State<Arc<AppContainer>>,
            Path(provider): Path<String>,
            Query(params): Query<CallbackParams>,
        ) -> ApiResult<OAuthTokenResponse> {
            let usecase = provider_usecase(&container, &provider)?;

            let user = usecase
                .callback(
                    &mut container.db.handle(),
                    SocialCallbackCommand {
                        code: params.code,
                        state: params.state,
                    },
                )
                .await?;

            let auth_provider = parse_provider(&provider)?;
            let provider_uid = user
                .identities
                .iter()
                .find(|i| i.provider == auth_provider)
                .map(|i| i.provider_uid.clone())
                .ok_or_else(|| shinespark::Error::IllegalState("identity not found".into()))?;

            let pair = container
                .jwt_ident_usecase
                .login(
                    &mut container.db.handle(),
                    LoginCommand::Social {
                        provider: auth_provider,
                        provider_uid,
                    },
                )
                .await?;

            Ok(ApiResponse::new(OAuthTokenResponse {
                access_token: pair.access_token,
                refresh_token: pair.refresh_token,
            }))
        }

        fn provider_usecase(
            container: &Arc<AppContainer>,
            provider: &str,
        ) -> Result<Arc<dyn shinespark_identity::usecases::SocialLoginUsecase>, ApiError> {
            match provider {
                "google" => Ok(container.google_login_usecase.clone()),
                _ => Err(shinespark::Error::NotImplemented.into()),
            }
        }

        fn parse_provider(provider: &str) -> shinespark::Result<AuthProvider> {
            match provider {
                "google" => Ok(AuthProvider::Google),
                "apple" => Ok(AuthProvider::Apple),
                _ => Err(shinespark::Error::NotFound),
            }
        }

        pub fn routes() -> Router<Arc<AppContainer>> {
            Router::new()
                .route(
                    "/identity/oauth2/{provider}/login",
                    axum::routing::get(login),
                )
                .route(
                    "/identity/oauth2/{provider}/callback",
                    axum::routing::get(callback),
                )
        }
    }

    pub fn routes() -> Router<Arc<AppContainer>> {
        Router::new().merge(session::routes()).merge(jwt::routes()).merge(oauth2::routes())
    }
}
