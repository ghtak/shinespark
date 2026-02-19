pub struct PostgresUserRepository;

#[async_trait::async_trait]
impl crate::repository::UserRepository for PostgresUserRepository {
    type Database = sqlx::Postgres;

    async fn find_user(
        &self,
        e: impl sqlx::Executor<'_, Database = Self::Database>,
    ) -> shinespark::Result<Vec<crate::entity::User>> {
        let users: Vec<crate::entity::User> =
            sqlx::query_as("select * from ss_id_users")
                .fetch_all(e)
                .await
                .map_err(|e| shinespark::Error::Unexpected(e.into()))?;
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        infra::user_repository_impl::PostgresUserRepository,
        repository::UserRepository,
    };

    #[tokio::test]
    async fn test_find_user() {
        let pool = sqlx::pool::PoolOptions::<sqlx::Postgres>::new()
            .max_connections(1)
            .connect("postgres://username:password@localhost:5432/shinespark")
            .await
            .expect("");

        let user_repo = PostgresUserRepository;
        user_repo.find_user(&pool).await.expect("");
    }
}
