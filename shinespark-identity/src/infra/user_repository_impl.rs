use shinespark::db::AsExecutor;

pub struct PostgresUserRepository;

#[async_trait::async_trait]
impl crate::repository::UserRepository for PostgresUserRepository {
    async fn find_user(
        &self,
        h: &mut shinespark::db::AppDbHandle<'_>,
    ) -> shinespark::Result<Vec<crate::entity::User>> {
        let users: Vec<crate::entity::User> =
            sqlx::query_as("select * from ss_id_users")
                .fetch_all(h.as_executor())
                .await
                .map_err(|e| shinespark::Error::Unexpected(e.into()))?;
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use shinespark::db::AppDbHandle;

    use crate::{
        infra::user_repository_impl::PostgresUserRepository,
        repository::UserRepository,
    };

    async fn dyn_repo_call_test(
        repo: Arc<dyn UserRepository>,
        handle: &mut AppDbHandle<'_>,
    ) {
        let _ = repo.find_user(handle).await.expect("");
    }

    #[tokio::test]
    async fn test_find_user() {
        let pool = sqlx::pool::PoolOptions::<sqlx::Postgres>::new()
            .max_connections(1)
            .connect("postgres://username:password@localhost:5432/shinespark")
            .await
            .expect("");

        let user_repo = PostgresUserRepository;
        let mut handle = shinespark::db::Handle::Pool(pool);
        user_repo.find_user(&mut handle).await.expect("");

        let dyn_repo: Arc<dyn UserRepository> = Arc::new(user_repo);
        dyn_repo_call_test(dyn_repo, &mut handle).await;
    }
}
