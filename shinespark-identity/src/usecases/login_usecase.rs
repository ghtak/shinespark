use crate::entities::UserAggregate;

// ==========================================
// 1. LoginUsecase Cqrs
// ==========================================
#[derive(Debug)]
pub enum LoginCommand {
    Local {
        email: String,
        password: String,
    },
    Social {
        provider: crate::entities::AuthProvider,
        provider_uid: String,
    },
}

// ==========================================
// 2. LoginUsecase Trait
// ==========================================
// 사용자의 자격 증명을 이용한 시스템 인증(Authentication) 처리에 집중합니다.
#[async_trait::async_trait]
pub trait LoginUsecase: Send + Sync + 'static {
    async fn login(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: LoginCommand,
    ) -> shinespark::Result<UserAggregate>;
}
