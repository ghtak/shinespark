#[async_trait::async_trait]
pub trait UserRepository: Sync + Send {
    type Database: sqlx::Database;

    async fn find_user(
        &self,
        e: impl sqlx::Executor<'_, Database = Self::Database>,
    ) -> shinespark::Result<Vec<crate::entity::User>>;
}
