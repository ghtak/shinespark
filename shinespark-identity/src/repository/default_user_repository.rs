use shinespark::db::{SqlComposer, SqlStatement};

use crate::entity::{User, UserIdentity, UserWithRoles};
use crate::repository::{Query, UserRepository};
use crate::service::{FindUserQuery, UpdateUserCommand};

pub struct DefaultUserRepository {}

impl DefaultUserRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl UserRepository for DefaultUserRepository {
    async fn create_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user: User,
    ) -> shinespark::Result<User> {
        let user = Query::CreateUser
            .as_query_as::<User>()
            .bind(&user.uid)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.status.as_str())
            .fetch_one(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(user)
    }

    async fn create_identity(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_identity: UserIdentity,
    ) -> shinespark::Result<UserIdentity> {
        let user_identity = Query::CreateIdentity
            .as_query_as::<UserIdentity>()
            .bind(&user_identity.user_id)
            .bind(&user_identity.provider.as_str())
            .bind(&user_identity.provider_uid)
            .bind(&user_identity.credential_hash)
            .fetch_one(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(user_identity)
    }

    async fn find_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        query: FindUserQuery,
    ) -> shinespark::Result<Option<UserWithRoles>> {
        #[derive(sqlx::FromRow)]
        struct _Row {
            #[sqlx(flatten)]
            pub user: User,
            pub role_ids: sqlx::types::Json<Vec<i64>>,
        }
        let mut b = Query::FindUser.as_builder();
        query.compose(&mut b)?;
        let row = b
            .build_query_as::<_Row>()
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(row.map(|r| UserWithRoles {
            user: r.user,
            role_ids: r.role_ids.0,
        }))
    }

    async fn update_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: UpdateUserCommand,
    ) -> shinespark::Result<User> {
        let mut builder = Query::UpdateUser.as_builder();
        command.compose(&mut builder)?;
        let user = builder
            .build_query_as::<User>()
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        if user.is_none() {
            return Err(shinespark::Error::NotFound);
        }
        Ok(user.unwrap())
    }
}
