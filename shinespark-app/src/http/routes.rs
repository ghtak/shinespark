pub mod identity {
    pub mod dto {

        #[derive(Debug, serde::Deserialize)]
        pub struct LoginRequest {
            pub email: String,
            pub password: String,
        }
    }

    use std::sync::Arc;

    use axum::{Json, Router, extract::State};
    use shinespark_identity::{entities::UserAggregate, usecases::LoginCommand};

    use crate::{
        AppContainer,
        http::{ApiResponse, ApiResult},
    };

    async fn login(
        State(container): State<Arc<AppContainer>>,
        Json(command): Json<dto::LoginRequest>,
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
        Ok(ApiResponse::new(user_aggregate))
    }

    pub fn routes() -> Router<Arc<AppContainer>> {
        Router::new().route("/identity/login", axum::routing::post(login))
    }
}
