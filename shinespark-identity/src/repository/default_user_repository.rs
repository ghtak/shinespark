use chrono::Utc;

use crate::entity::{Status, User, UserIdentity, UserWithRoles};
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
            user.status,
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
            user_identity.provider,
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

        let user = sqlx::query_as::<_, _UserWithRoles>(
            r#"
            SELECT u.*,
                COALESCE(
                    json_agg(r.role_id) FILTER (WHERE r.role_id IS NOT NULL),
                    '[]'::json
                ) as role_ids
            FROM shs_iam_user u
                LEFT JOIN shs_iam_user_role r ON u.id = r.user_id
            WHERE 1=1
                AND ($1::bigint IS NULL OR u.id = $1)
                AND ($2::uuid IS NULL OR u.uid = $2)
                AND ($3::text IS NULL OR u.email = $3)
            GROUP BY u.id
            "#,
        )
        .bind(query.id)
        .bind(query.uid)
        .bind(query.email)
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
            r#" UPDATE shs_iam_user
                SET status = $1, updated_at = $2
                WHERE id = $3 "#,
            Status::Deleted.as_str(),
            Utc::now(),
            user_id,
        )
        .execute(handle.inner())
        .await
        .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;

        Ok(())
    }
}
