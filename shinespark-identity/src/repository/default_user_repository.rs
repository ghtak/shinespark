use shinespark::db::QueryFilter;

use crate::entity::{User, UserIdentity, UserWithRoles};
use crate::repository::UserRepository;
use crate::service::FindUserQuery;

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
        let sql = include_str!("../../sql/user_repository/create_user.sql");
        let user = sqlx::query_as::<_, User>(sql)
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
        let sql = include_str!("../../sql/user_repository/create_identity.sql");
        let user_identity = sqlx::query_as::<_, UserIdentity>(sql)
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
        let sql = include_str!("../../sql/user_repository/find_user.sql");
        let mut b = sqlx::QueryBuilder::<shinespark::db::Driver>::new(sql);
        query.apply(&mut b)?;
        b.push(" GROUP BY u.id");
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

    async fn delete_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
    ) -> shinespark::Result<()> {
        let sql = include_str!("../../sql/user_repository/delete_user.sql");
        let user = sqlx::query_as::<_, User>(sql)
            .bind(&crate::entity::UserStatus::Deleted.as_str())
            .bind(&user_id)
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        if user.is_none() {
            return Err(shinespark::Error::NotFound);
        }
        Ok(())
    }
}
