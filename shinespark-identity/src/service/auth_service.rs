use crate::entity::{UserWithIdentities, UserWithRoles};
use shinespark::Result;

#[derive(Debug)]
pub enum LoginCommand {
    Local {
        email: String,
        password: String,
    },
    Social {
        provider: crate::entity::AuthProvider,
        provider_uid: String,
    },
}

#[async_trait::async_trait]
pub trait AuthService: Send + Sync + 'static {
    /// 회원가입(SignUp)
    /// 서비스 계층 내에서 UserService를 호출하여 사용자를 생성하고,
    /// 필요한 경우 초기 권한(Rbac)을 할당하는 등의 복합 워크플로우를 처리합니다.
    async fn sign_up(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: crate::service::CreateUserCommand,
    ) -> Result<UserWithIdentities>;

    // /// 로그인(Login)
    // /// 사용자의 인증 정보를 확인(PasswordService 활용 등)하고,
    // /// 권한 정보(UserWithRoles)를 가져와 반환합니다.
    // async fn login(
    //     &self,
    //     handle: &mut shinespark::db::Handle<'_>,
    //     command: LoginCommand,
    // ) -> Result<UserWithRoles>;

    // /// 로그아웃(Logout)
    // /// 로그인 이력 저장 등의 기능을 담당합니다.
    // async fn logout(&self, handle: &mut shinespark::db::Handle<'_>, user_id: i64) -> Result<()>;
}
