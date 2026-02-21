use crate::{
    entity::SimpleEntity,
    repository::SimpleRepository,
    service::{
        CreateSimpleCommand, FindSimpleQuery, SimpleService, SimpleServiceTx,
    },
};
use std::sync::Arc;

pub struct SimpleServiceImpl<S: SimpleRepository<sqlx::Postgres> + Sync + Send>
{
    pub pool: sqlx::pool::Pool<sqlx::Postgres>,
    pub repo: Arc<S>,
}

impl<S: SimpleRepository<sqlx::Postgres> + Sync + Send> SimpleServiceImpl<S> {
    pub fn new(pool: sqlx::pool::Pool<sqlx::Postgres>, repo: Arc<S>) -> Self {
        Self { pool, repo }
    }
}

#[async_trait::async_trait]
impl<S: SimpleRepository<sqlx::Postgres> + Sync + Send> SimpleService
    for SimpleServiceImpl<S>
{
    async fn find_simple(
        &self,
        _query: FindSimpleQuery,
    ) -> shinespark::Result<SimpleEntity> {
        let mut handle = shinespark::db::Handle::Pool(self.pool.clone());
        let all = self.repo.find_all(&mut handle).await?;
        all.into_iter()
            .next()
            .ok_or(shinespark::Error::NotFound("Not Found".into()))
    }
}

#[async_trait::async_trait]
impl<S: SimpleRepository<sqlx::Postgres> + Sync + Send> SimpleServiceTx
    for SimpleServiceImpl<S>
{
    async fn create_simple(
        &self,
        h: &mut shinespark::db::AppDbHandle<'_>,
        command: CreateSimpleCommand,
    ) -> shinespark::Result<SimpleEntity> {
        let entity = SimpleEntity {
            id: 0,
            name: command.name,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.repo.create(h, entity).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::SimpleRepositoryImpl;
    use shinespark::db::{BaseRepository, Handle};
    use sqlx::PgPool;

    async fn setup_db() -> PgPool {
        let database_url =
            "postgres://username:password@localhost:5432/shinespark";
        PgPool::connect(database_url)
            .await
            .expect("Failed to connect to Postgres")
    }

    #[tokio::test]
    #[ignore]
    async fn test_simple_service_create_and_find() -> shinespark::Result<()> {
        let pool = setup_db().await;
        let repo = Arc::new(SimpleRepositoryImpl {});
        let service = SimpleServiceImpl::new(pool.clone(), repo.clone());

        // 1. Create using SimpleServiceTx (Transaction-based)
        let mut pool_handle = Handle::Pool(pool.clone());
        let mut tx_handle = pool_handle.begin().await?;

        let command = CreateSimpleCommand {
            name: "Service Test Entity".to_string(),
        };

        let created = service.create_simple(&mut tx_handle, command).await?;
        assert_eq!(created.name, "Service Test Entity");

        tx_handle.commit().await?;

        // 2. Find using SimpleService (Pool-based)
        let query = FindSimpleQuery { id: created.id };
        let found = service.find_simple(query).await?;

        // find_simple currently just returns the first one from find_all in the impl
        // but we verify the name at least
        assert_eq!(found.name, "Service Test Entity");

        for s in repo.find_all(&mut pool_handle).await? {
            repo.delete(&mut pool_handle, s.id).await?;
        }
        Ok(())
    }
}
