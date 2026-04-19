use std::sync::Arc;

use serde::Deserialize;
use shinespark::config::GoogleLoginConfig;

use crate::entities::{AuthProvider, UserAggregate, UserStatus};
use crate::usecases::{
    CreateUserCommand, FindUserQuery, InitialCredentials, LoginCommand, LoginUsecase, RbacUsecase,
    SocialCallbackCommand, SocialLoginCommand, SocialLoginUsecase, UserUsecase,
};

#[derive(Deserialize, Debug)]
struct GoogleTokenResponse {
    id_token: String,
}

#[derive(Deserialize, Debug)]
struct GoogleIdTokenClaims {
    sub: String,
    email: Option<String>,
    name: Option<String>,
}

pub struct DefaultGoogleLoginUsecase {
    config: GoogleLoginConfig,
    http: reqwest::Client,
    user_usecase: Arc<dyn UserUsecase>,
    login_usecase: Arc<dyn LoginUsecase>,
    rbac_usecase: Arc<dyn RbacUsecase>,
}

impl DefaultGoogleLoginUsecase {
    pub fn new(
        config: GoogleLoginConfig,
        user_usecase: Arc<dyn UserUsecase>,
        login_usecase: Arc<dyn LoginUsecase>,
        rbac_usecase: Arc<dyn RbacUsecase>,
    ) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
            user_usecase,
            login_usecase,
            rbac_usecase,
        }
    }
}

#[async_trait::async_trait]
impl SocialLoginUsecase for DefaultGoogleLoginUsecase {
    async fn login(&self, cmd: SocialLoginCommand) -> shinespark::Result<String> {
        let url = format!(
            "https://accounts.google.com/o/oauth2/v2/auth?response_type=code\
            &client_id={}&redirect_uri={}&scope={}&state={}&access_type=offline&prompt=consent",
            self.config.client_id.as_str(),
            self.config.redirect_uri.as_str(),
            self.config.scope.as_str(),
            cmd.state.as_str(),
        );
        Ok(url)
    }

    async fn callback(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        cmd: SocialCallbackCommand,
    ) -> shinespark::Result<UserAggregate> {
        let token_resp = self
            .http
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("code", cmd.code.as_str()),
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
                ("redirect_uri", self.config.redirect_uri.as_str()),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|e| shinespark::Error::Internal(anyhow::anyhow!(e)))?
            .json::<GoogleTokenResponse>()
            .await
            .map_err(|e| shinespark::Error::Internal(anyhow::anyhow!(e)))?;

        let claims = decode_id_token(self.config.client_id.as_str(), &token_resp.id_token).await?;

        let login_result = self
            .login_usecase
            .login(
                handle,
                LoginCommand::Social {
                    provider: AuthProvider::Google,
                    provider_uid: claims.sub.clone(),
                },
            )
            .await;

        match login_result {
            Ok(user) => Ok(user),
            Err(shinespark::Error::NotFound) => {
                let created = self
                    .user_usecase
                    .create_user(
                        handle,
                        CreateUserCommand {
                            name: claims.name.unwrap_or_else(|| "Google User".to_string()),
                            email: claims.email.unwrap_or_default(),
                            credentials: InitialCredentials::Social {
                                provider: AuthProvider::Google,
                                provider_uid: claims.sub,
                            },
                            status: UserStatus::Active,
                        },
                    )
                    .await?;
                self.rbac_usecase.assign_role_to_user(handle, created.user.id, "user").await?;
                self.user_usecase
                    .find_user(handle, FindUserQuery::new().id(created.user.id))
                    .await?
                    .ok_or_else(|| {
                        shinespark::Error::IllegalState("user not found after creation".into())
                    })
            }
            Err(e) => Err(e),
        }
    }
}

async fn decode_id_token(
    client_id: &str,
    id_token: &str,
) -> shinespark::Result<GoogleIdTokenClaims> {
    // let mut validation = Validation::new(Algorithm::RS256);
    // validation.insecure_disable_signature_validation();
    // validation.validate_exp = false;

    // decode::<GoogleIdTokenClaims>(id_token, &DecodingKey::from_secret(b""), &validation)
    //     .map(|d| d.claims)
    //     .map_err(|e| {
    //         shinespark::Error::Internal(anyhow::anyhow!(e).context("id_token decode failed"))
    //     })
    let client = google_oauth::AsyncClient::new(client_id.to_string());
    let id_token = client.validate_id_token(id_token).await.map_err(|e| {
        shinespark::Error::Internal(anyhow::anyhow!(e).context("id_token decode failed"))
    })?;
    Ok(GoogleIdTokenClaims {
        sub: id_token.sub,
        email: id_token.email,
        name: id_token.name,
    })
}
