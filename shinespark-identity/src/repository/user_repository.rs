use crate::{
    entity::{User, UserAggregate, UserIdentity},
    service::{FindUserQuery, UpdateUserCommand},
};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user: User,
    ) -> shinespark::Result<User>;

    async fn create_identity(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_identity: UserIdentity,
    ) -> shinespark::Result<UserIdentity>;

    async fn find_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        query: FindUserQuery,
    ) -> shinespark::Result<Option<UserAggregate>>;

    async fn find_user_by_identity(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        provider: crate::entity::AuthProvider,
        provider_uid: String,
    ) -> shinespark::Result<Option<UserAggregate>>;

    async fn update_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: UpdateUserCommand,
    ) -> shinespark::Result<User>;
}
