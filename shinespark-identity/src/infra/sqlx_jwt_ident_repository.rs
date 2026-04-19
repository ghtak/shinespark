use chrono::{DateTime, Utc};
use shinespark::db::SqlStatement;

use crate::repositories::{JwtIdentRepository, RefreshTokenRow};

pub struct SqlxJwtIdentRepository {}

impl SqlxJwtIdentRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl JwtIdentRepository for SqlxJwtIdentRepository {
    async fn save_refresh_token(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_uid: &str,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> shinespark::Result<()> {
        let uid = uuid::Uuid::parse_str(user_uid).map_err(|e| {
            shinespark::Error::Internal(anyhow::anyhow!(e).context("invalid user_uid"))
        })?;

        r#"
        INSERT INTO
            shs_iam_refresh_token (user_uid, token_hash, expires_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (token_hash) DO NOTHING
        "#
        .as_query_as::<(i64,)>()
        .bind(uid)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_optional(handle.inner())
        .await
        .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;

        Ok(())
    }

    async fn find_refresh_token(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        token_hash: &str,
    ) -> shinespark::Result<Option<RefreshTokenRow>> {
        r#"
        SELECT
            id, user_uid, token_hash, expires_at, created_at
        FROM
            shs_iam_refresh_token
        WHERE 1=1
            AND token_hash = $1
            AND expires_at > NOW()
        "#
        .as_query_as::<RefreshTokenRow>()
        .bind(token_hash)
        .fetch_optional(handle.inner())
        .await
        .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))
    }

    async fn delete_by_user_uid(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_uid: &str,
    ) -> shinespark::Result<()> {
        let uid = uuid::Uuid::parse_str(user_uid).map_err(|e| {
            shinespark::Error::Internal(anyhow::anyhow!(e).context("invalid user_uid"))
        })?;

        "DELETE FROM shs_iam_refresh_token WHERE user_uid = $1"
            .as_query_as::<(i64,)>()
            .bind(uid)
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::JwtIdentRepository;

    #[tokio::test]
    #[ignore]
    async fn test_save_and_find_refresh_token() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let repo = SqlxJwtIdentRepository::new();
        let mut handle = db.handle();

        let uid = "00000000-0000-0000-0000-000000000001";
        let hash = "testhash1234";
        let expires_at = Utc::now() + chrono::Duration::hours(24);

        repo.save_refresh_token(&mut handle, uid, hash, expires_at).await.unwrap();
        let row = repo.find_refresh_token(&mut handle, hash).await.unwrap();
        assert!(row.is_some());
        assert_eq!(row.unwrap().token_hash, hash);
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_by_user_uid() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let repo = SqlxJwtIdentRepository::new();
        let mut handle = db.handle();

        let uid = "00000000-0000-0000-0000-000000000002";
        let hash = "testhash5678";
        let expires_at = Utc::now() + chrono::Duration::hours(24);

        repo.save_refresh_token(&mut handle, uid, hash, expires_at).await.unwrap();
        repo.delete_by_user_uid(&mut handle, uid).await.unwrap();
        let row = repo.find_refresh_token(&mut handle, hash).await.unwrap();
        assert!(row.is_none());
    }
}
