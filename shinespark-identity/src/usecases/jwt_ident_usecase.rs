use crate::infra::jwt_service::JwtTokenPair;
use crate::usecases::LoginCommand;

#[async_trait::async_trait]
pub trait JwtIdentUsecase: Send + Sync + 'static {
    async fn login(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: LoginCommand,
    ) -> shinespark::Result<JwtTokenPair>;

    async fn logout(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_uid: &str,
    ) -> shinespark::Result<()>;

    async fn refresh(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        refresh_token: &str,
    ) -> shinespark::Result<JwtTokenPair>;
}
