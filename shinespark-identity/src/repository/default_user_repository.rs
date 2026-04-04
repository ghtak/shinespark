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
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO shs_iam_user (uid, name, email, status)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (email) DO UPDATE SET
                name = EXCLUDED.name,
                status = EXCLUDED.status
            RETURNING *
            "#,
            user.uid,
            user.name,
            user.email,
            user.status.as_str(),
        )
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
        let user_identity = sqlx::query_as!(
            UserIdentity,
            r#"
            INSERT INTO shs_iam_user_identity (user_id, provider, provider_uid, credential_hash)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, provider, provider_uid) DO UPDATE SET
                credential_hash = COALESCE(EXCLUDED.credential_hash, shs_iam_user_identity.credential_hash),
                updated_at = NOW()
            RETURNING *
            "#,
            user_identity.user_id,
            user_identity.provider.as_str(),
            user_identity.provider_uid,
            user_identity.credential_hash
        )
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
        struct _UserWithRoles {
            #[sqlx(flatten)]
            pub user: User,
            pub role_ids: sqlx::types::Json<Vec<i64>>,
        }

        let find_user_sql = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/sql/user_repository/find_user.sql"
        ));

        let mut b = sqlx::QueryBuilder::<shinespark::db::Driver>::new(find_user_sql);

        query.apply(&mut b)?;

        b.push(" GROUP BY u.id");

        let user = b
            .build_query_as::<_UserWithRoles>()
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;

        Ok(user.map(|user| UserWithRoles {
            user: user.user,
            role_ids: user.role_ids.0,
        }))
    }

    async fn delete_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query!(
            r#"
            UPDATE shs_iam_user
            SET status = $1, updated_at = now()
            WHERE id = $2
            "#,
            crate::entity::UserStatus::Deleted.as_str(),
            user_id,
        )
        .execute(handle.inner())
        .await
        .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;

        Ok(())
    }
}
