mod handle;
mod repository;

pub use handle::*;
pub use repository::*;

pub type PostgresHandle<'c> = Handle<'c, sqlx::Postgres>;
pub type MysqlHandle<'c> = Handle<'c, sqlx::MySql>;
pub type SqliteHandle<'c> = Handle<'c, sqlx::Sqlite>;

// Application default db driver
// if required change this
pub type AppDbDriver = sqlx::Postgres;
pub type AppDbHandle<'c> = Handle<'c, AppDbDriver>;

pub fn map_err(e: sqlx::Error) -> crate::Error {
    crate::Error::Database(anyhow::Error::new(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn run_generic_handle_test<DB>(
        pool: sqlx::Pool<DB>,
        dummy_query: &str,
    ) where
        DB: sqlx::Database,
        for<'e> &'e mut <DB as sqlx::Database>::Connection:
            sqlx::Executor<'e, Database = DB>,
        for<'q> <DB as sqlx::Database>::Arguments<'q>:
            sqlx::IntoArguments<'q, DB>,
    {
        // 1. Test Pool handle
        {
            let mut handle = Handle::Pool(pool.clone());
            let exec = handle.as_executor();
            sqlx::query(dummy_query)
                .execute(exec)
                .await
                .expect("Failed to execute query via Pool handle");
        }

        // 2. Test Transaction handle
        {
            let mut pool_handle = Handle::Pool(pool.clone());
            let mut tx_handle =
                pool_handle.begin().await.expect("Failed to begin transaction");

            {
                let exec = tx_handle.as_executor();
                sqlx::query(dummy_query)
                    .execute(exec)
                    .await
                    .expect("Failed to execute query via Transaction handle");
            }

            tx_handle.commit().await.expect("Failed to commit transaction");
        }

        // 3. Test Connection handle
        {
            let conn =
                pool.acquire().await.expect("Failed to acquire connection");
            let mut handle = Handle::Connection(conn);
            let exec = handle.as_executor();
            sqlx::query(dummy_query)
                .execute(exec)
                .await
                .expect("Failed to execute query via Connection handle");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_sqlite_handle() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_generic_handle_test(pool, "SELECT 1").await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_postgres_handle() {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for postgres test");
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        run_generic_handle_test(pool, "SELECT 1").await;
    }
}
