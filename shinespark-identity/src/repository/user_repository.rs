use crate::{
    entity::{User, UserIdentity, UserWithRoles},
    service::{FindUserQuery, UpdateUserCommand},
};

#[async_trait::async_trait]
pub trait UserRepository: Sync + Send {
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
    ) -> shinespark::Result<Option<UserWithRoles>>;

    async fn delete_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
    ) -> shinespark::Result<User>;

    async fn update_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: UpdateUserCommand,
    ) -> shinespark::Result<User>;
}
