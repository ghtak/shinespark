use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshTokenRow {
    pub id: i64,
    pub user_uid: uuid::Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait JwtIdentRepository: Send + Sync + 'static {
    async fn save_refresh_token(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_uid: &str,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> shinespark::Result<()>;

    async fn find_refresh_token(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        token_hash: &str,
    ) -> shinespark::Result<Option<RefreshTokenRow>>;

    async fn delete_by_user_uid(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_uid: &str,
    ) -> shinespark::Result<()>;
}
