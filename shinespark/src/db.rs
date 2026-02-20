use futures_core::{future::BoxFuture, stream::BoxStream};

use sqlx::Acquire;

impl From<sqlx::Error> for crate::Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(anyhow::Error::new(value))
    }
}

#[derive(Debug)]
pub enum Handle<'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut <DB as sqlx::Database>::Connection:
        sqlx::Executor<'e, Database = DB>,
{
    Pool(sqlx::Pool<DB>),
    Transaction(sqlx::Transaction<'c, DB>),
    Connection(sqlx::pool::PoolConnection<DB>),
}

impl<'c, DB> Handle<'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut <DB as sqlx::Database>::Connection:
        sqlx::Executor<'e, Database = DB>,
{
    pub async fn begin(&mut self) -> crate::Result<Handle<'_, DB>> {
        let tx = match self {
            Handle::Pool(pool) => pool.begin().await,
            Handle::Transaction(tx) => tx.begin().await,
            Handle::Connection(conn) => conn.begin().await,
        }?;
        Ok(Handle::Transaction(tx))
    }

    pub async fn commit(self) -> crate::Result<()> {
        match self {
            Handle::Pool(_) => Ok(()),
            Handle::Transaction(tx) => tx.commit().await.map_err(Into::into),
            Handle::Connection(_) => Ok(()),
        }
    }

    pub async fn rollback(self) -> crate::Result<()> {
        match self {
            Handle::Pool(_) => Ok(()),
            Handle::Transaction(tx) => tx.rollback().await.map_err(Into::into),
            Handle::Connection(_) => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct ExecutorImpl<'h, 'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut <DB as sqlx::Database>::Connection:
        sqlx::Executor<'e, Database = DB>,
{
    pub handle: &'h mut Handle<'c, DB>,
}

pub trait AsExecutor {
    type Executor<'h>: sqlx::Executor<'h>
    where
        Self: 'h;

    fn as_executor<'h>(&'h mut self) -> Self::Executor<'h>;
}

impl<'c, DB> AsExecutor for Handle<'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut <DB as sqlx::Database>::Connection:
        sqlx::Executor<'e, Database = DB>,
{
    type Executor<'h>
        = ExecutorImpl<'h, 'c, DB>
    where
        'c: 'h;

    fn as_executor<'h>(&'h mut self) -> Self::Executor<'h> {
        ExecutorImpl { handle: self }
    }
}

impl<'h, 'c, DB> sqlx::Executor<'h> for ExecutorImpl<'h, 'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut <DB as sqlx::Database>::Connection:
        sqlx::Executor<'e, Database = DB>,
{
    type Database = DB;

    /// Execute multiple queries and return the generated results as a stream
    /// from each query, in a stream.
    fn fetch_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxStream<
        'e,
        Result<
            sqlx::Either<
                <Self::Database as sqlx::Database>::QueryResult,
                <Self::Database as sqlx::Database>::Row,
            >,
            sqlx::Error,
        >,
    >
    where
        'c: 'e,
        'h: 'e,
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        match self.handle {
            Handle::Pool(pool) => pool.fetch_many(query),
            Handle::Transaction(tx) => tx.fetch_many(query),
            Handle::Connection(conn) => conn.fetch_many(query),
        }
    }

    /// Execute the query and returns at most one row.
    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<
        'e,
        Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>,
    >
    where
        'c: 'e,
        'h: 'e,
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        match self.handle {
            Handle::Pool(pool) => pool.fetch_optional(query),
            Handle::Transaction(tx) => tx.fetch_optional(query),
            Handle::Connection(conn) => conn.fetch_optional(query),
        }
    }

    /// Prepare the SQL query, with parameter type information, to inspect the
    /// type information about its parameters and results.
    ///
    /// Only some database drivers (PostgreSQL, MSSQL) can take advantage of
    /// this extra information to influence parameter type inference.
    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
    ) -> BoxFuture<
        'e,
        Result<<Self::Database as sqlx::Database>::Statement<'q>, sqlx::Error>,
    >
    where
        'c: 'e,
        'h: 'e,
    {
        match self.handle {
            Handle::Pool(pool) => pool.prepare_with(sql, parameters),
            Handle::Transaction(tx) => tx.prepare_with(sql, parameters),
            Handle::Connection(conn) => conn.prepare_with(sql, parameters),
        }
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
    where
        'c: 'e,
        'h: 'e,
    {
        match self.handle {
            Handle::Pool(pool) => pool.describe(sql),
            Handle::Transaction(tx) => tx.describe(sql),
            Handle::Connection(conn) => conn.describe(sql),
        }
    }
}

pub type PostgresHandle<'c> = Handle<'c, sqlx::Postgres>;
pub type MysqlHandle<'c> = Handle<'c, sqlx::MySql>;
pub type SqliteHandle<'c> = Handle<'c, sqlx::Sqlite>;

pub type AppDbHandle<'c> = PostgresHandle<'c>;

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
