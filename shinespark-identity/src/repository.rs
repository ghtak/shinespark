use crate::entity::{AuthProvider, User, UserIdentity};
use shinespark::db::AppDbHandle;

#[async_trait::async_trait]
pub trait UserRepository: Sync + Send {
    async fn find_user(
        &self,
        h: &mut AppDbHandle<'_>,
    ) -> shinespark::Result<Vec<User>>;

    async fn insert_user(
        &self,
        h: &mut AppDbHandle<'_>,
        name: Option<&str>,
        email: &str,
    ) -> shinespark::Result<User>;
}

#[async_trait::async_trait]
pub trait UserIdentityRepository: Sync + Send {
    async fn insert_identity(
        &self,
        h: &mut AppDbHandle<'_>,
        user_id: i64,
        provider: AuthProvider,
        provider_user_id: &str,
        credential_hash: &str,
    ) -> shinespark::Result<UserIdentity>;
}
