use shinespark::db::{AsExecutor, BaseRepository};

use crate::{entity::SimpleEntity, repository::SimpleRepository};

pub struct SimpleRepositoryImpl {}

#[async_trait::async_trait]
impl BaseRepository<SimpleEntity, sqlx::Postgres> for SimpleRepositoryImpl {
    async fn create(
        &self,
        h: &mut shinespark::db::Handle<'_, sqlx::Postgres>,
        entity: crate::entity::SimpleEntity,
    ) -> shinespark::Result<crate::entity::SimpleEntity> {
        sqlx::query_as("INSERT INTO simple (name) VALUES ($1) RETURNING *")
            .bind(entity.name)
            .fetch_one(h.as_executor())
            .await
            .map_err(shinespark::db::map_err)
    }

    async fn find_by_id(
        &self,
        h: &mut shinespark::db::Handle<'_, sqlx::Postgres>,
        id: i64,
    ) -> shinespark::Result<Option<crate::entity::SimpleEntity>> {
        sqlx::query_as("SELECT * FROM simple WHERE id = $1")
            .bind(id)
            .fetch_optional(h.as_executor())
            .await
            .map_err(shinespark::db::map_err)
    }

    async fn find_all(
        &self,
        h: &mut shinespark::db::Handle<'_, sqlx::Postgres>,
    ) -> shinespark::Result<Vec<crate::entity::SimpleEntity>> {
        sqlx::query_as("SELECT * FROM simple")
            .fetch_all(h.as_executor())
            .await
            .map_err(shinespark::db::map_err)
    }

    async fn update(
        &self,
        h: &mut shinespark::db::Handle<'_, sqlx::Postgres>,
        entity: crate::entity::SimpleEntity,
    ) -> shinespark::Result<crate::entity::SimpleEntity> {
        sqlx::query_as("UPDATE simple SET name = $1 WHERE id = $2 RETURNING *")
            .bind(entity.name)
            .bind(entity.id)
            .fetch_one(h.as_executor())
            .await
            .map_err(shinespark::db::map_err)
    }

    async fn delete(
        &self,
        h: &mut shinespark::db::Handle<'_, sqlx::Postgres>,
        id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query("DELETE FROM simple WHERE id = $1")
            .bind(id)
            .execute(h.as_executor())
            .await
            .map_err(shinespark::db::map_err)?;
        Ok(())
    }
}

impl SimpleRepository<sqlx::Postgres> for SimpleRepositoryImpl {}

#[cfg(test)]
mod tests {
    use super::*;
    use shinespark::db::Handle;
    use sqlx::PgPool;

    async fn setup_db() -> PgPool {
        let database_url =
            "postgres://username:password@localhost:5432/shinespark";
        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to Postgres")
    }

    #[tokio::test]
    #[ignore]
    async fn test_simple_repository_crud() -> shinespark::Result<()> {
        let pool = setup_db().await;
        let mut handle = Handle::Pool(pool);
        let repo = SimpleRepositoryImpl {};

        // 1. Create
        let new_entity = SimpleEntity {
            id: 0, // Will be ignored by DB if serial/auto-inc, but Postgres usually needs RETURNING
            name: "Test Entity".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created = repo.create(&mut handle, new_entity.clone()).await?;
        assert_eq!(created.name, new_entity.name);
        let created_id = created.id;

        // 2. Find by ID
        let found = repo.find_by_id(&mut handle, created_id).await?;
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Entity");

        // 3. Find All
        let all = repo.find_all(&mut handle).await?;
        assert!(all.iter().any(|e| e.id == created_id));

        // 4. Update
        let mut to_update = created;
        to_update.name = "Updated Name".to_string();
        let updated = repo.update(&mut handle, to_update).await?;
        assert_eq!(updated.name, "Updated Name");

        // 5. Delete
        repo.delete(&mut handle, created_id).await?;
        let found_after_delete =
            repo.find_by_id(&mut handle, created_id).await?;
        assert!(found_after_delete.is_none());

        Ok(())
    }
}
