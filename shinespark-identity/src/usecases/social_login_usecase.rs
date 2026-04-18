use crate::entities::UserAggregate;

#[derive(Debug)]
pub struct SocialLoginCommand {
    pub state: String,
}

#[derive(Debug)]
pub struct SocialCallbackCommand {
    pub code: String,
    pub state: String,
}

#[async_trait::async_trait]
pub trait SocialLoginUsecase: Send + Sync + 'static {
    async fn login(&self, cmd: SocialLoginCommand) -> shinespark::Result<String>;

    async fn callback(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        cmd: SocialCallbackCommand,
    ) -> shinespark::Result<UserAggregate>;
}
