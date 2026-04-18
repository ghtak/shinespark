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

        use crate::{
            AppContainer,
            http::{ApiResponse, ApiResult},
        };

        async fn login(
            State(container): State<Arc<AppContainer>>,
            Json(command): Json<super::dto::LoginRequest>,
        ) -> ApiResult<()> {
            Ok(ApiResponse::new(()))
        }

        async fn logout() -> ApiResult<()> {
            Ok(ApiResponse::new(()))
        }

        async fn refresh_token() -> ApiResult<()> {
            Ok(ApiResponse::new(()))
        }

        pub fn routes() -> Router<Arc<AppContainer>> {
            Router::new()
                .route("/identity/jwt/login", axum::routing::post(login))
                .route("/identity/jwt/logout", axum::routing::post(logout))
                .route("/identity/jwt/refresh", axum::routing::post(refresh_token))
        }
    }

    pub fn routes() -> Router<Arc<AppContainer>> {
        Router::new().merge(session::routes()).merge(jwt::routes())
    }
}
