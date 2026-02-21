use crate::{
    repository::SimpleRepository,
    service::SimpleServiceTx,
    usecase::{
        CreateSimpleCommand, CreateSimpleUsecase, DeleteAllSimpleUsecase,
    },
};
use shinespark::db::Handle;
use std::sync::Arc;

pub struct SimpleUsecaseImpl<
    R: SimpleRepository<sqlx::Postgres> + Sync + Send,
    S: SimpleServiceTx + Sync + Send,
> {
    pub pool: sqlx::PgPool,
    pub repo: Arc<R>,
    pub service: Arc<S>,
}

impl<
    R: SimpleRepository<sqlx::Postgres> + Sync + Send,
    S: SimpleServiceTx + Sync + Send,
> SimpleUsecaseImpl<R, S>
{
    pub fn new(pool: sqlx::PgPool, repo: Arc<R>, service: Arc<S>) -> Self {
        Self {
            pool,
            repo,
            service,
        }
    }
}

#[async_trait::async_trait]
impl<
    R: SimpleRepository<sqlx::Postgres> + Sync + Send,
    S: SimpleServiceTx + Sync + Send,
> CreateSimpleUsecase for SimpleUsecaseImpl<R, S>
{
    async fn execute(
        &self,
        command: CreateSimpleCommand,
    ) -> shinespark::Result<()> {
        let mut pool_handle = Handle::Pool(self.pool.clone());
        let mut tx_handle = pool_handle.begin().await?;

        // Use the service to create the entity
        let service_command =
            crate::service::CreateSimpleCommand { name: command.name };

        self.service.create_simple(&mut tx_handle, service_command).await?;

        tx_handle.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl<
    R: SimpleRepository<sqlx::Postgres> + Sync + Send,
    S: SimpleServiceTx + Sync + Send,
> DeleteAllSimpleUsecase for SimpleUsecaseImpl<R, S>
{
    async fn execute(&self) -> shinespark::Result<()> {
        let mut pool_handle = Handle::Pool(self.pool.clone());
        let all = shinespark::db::BaseRepository::find_all(
            self.repo.as_ref(),
            &mut pool_handle,
        )
        .await?;

        let mut tx_handle = pool_handle.begin().await?;
        for entity in all {
            shinespark::db::BaseRepository::delete(
                self.repo.as_ref(),
                &mut tx_handle,
                entity.id,
            )
            .await?;
        }
        tx_handle.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::SimpleRepositoryImpl;
    use crate::service::SimpleServiceImpl;
    use shinespark::db::BaseRepository;
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
    async fn test_simple_usecase_create_and_delete_all()
    -> shinespark::Result<()> {
        let pool = setup_db().await;
        let repo = Arc::new(SimpleRepositoryImpl {});
        let service =
            Arc::new(SimpleServiceImpl::new(pool.clone(), repo.clone()));
        let usecase =
            SimpleUsecaseImpl::new(pool.clone(), repo.clone(), service);

        // 1. Create
        let command = CreateSimpleCommand {
            name: "Usecase Test Entity".to_string(),
        };
        CreateSimpleUsecase::execute(&usecase, command).await?;

        // 2. Verify
        let mut pool_handle = Handle::Pool(pool.clone());
        let all = repo.find_all(&mut pool_handle).await?;
        assert!(all.iter().any(|e| e.name == "Usecase Test Entity"));

        // 3. Delete All
        DeleteAllSimpleUsecase::execute(&usecase).await?;

        // 4. Verify Empty
        let all_after = repo.find_all(&mut pool_handle).await?;
        assert!(all_after.is_empty());

        Ok(())
    }
}
