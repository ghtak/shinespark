use chrono::{DateTime, Utc};
use shinespark::db::SqlStatement;

use crate::repositories::{JwtIdentRepository, RefreshTokenRow};

enum JwtQuery {
    SaveRefreshToken,
    FindRefreshToken,
    DeleteByUserUid,
}

impl SqlStatement for JwtQuery {
    fn as_str(&self) -> &'static str {
        match self {
            JwtQuery::SaveRefreshToken => include_str!("../../sql/jwt_repository/save_refresh_token.sql"),
            JwtQuery::FindRefreshToken => include_str!("../../sql/jwt_repository/find_refresh_token.sql"),
            JwtQuery::DeleteByUserUid => include_str!("../../sql/jwt_repository/delete_by_user_uid.sql"),
        }
    }
}

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
        let uid = uuid::Uuid::parse_str(user_uid)
            .map_err(|e| shinespark::Error::Internal(anyhow::anyhow!(e).context("invalid user_uid")))?;

        JwtQuery::SaveRefreshToken
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
        JwtQuery::FindRefreshToken
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
        let uid = uuid::Uuid::parse_str(user_uid)
            .map_err(|e| shinespark::Error::Internal(anyhow::anyhow!(e).context("invalid user_uid")))?;

        JwtQuery::DeleteByUserUid
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
