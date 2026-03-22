use futures_core::{future::BoxFuture, stream::BoxStream};

use sqlx::Acquire;

pub fn map_err(e: sqlx::Error) -> crate::Error {
    crate::Error::DatabaseError(anyhow::Error::new(e))
}

#[derive(Debug)]
pub enum BasicHandle<'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
{
    Pool(sqlx::Pool<DB>),
    Tx(sqlx::Transaction<'c, DB>),
    Conn(sqlx::pool::PoolConnection<DB>),
}

impl<'c, DB> BasicHandle<'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
{
    pub async fn begin(&mut self) -> crate::Result<BasicHandle<'_, DB>> {
        let tx = match self {
            BasicHandle::Pool(pool) => pool.begin().await,
            BasicHandle::Tx(tx) => tx.begin().await,
            BasicHandle::Conn(conn) => conn.begin().await,
        }
        .map_err(map_err)?;
        Ok(BasicHandle::Tx(tx))
    }

    pub async fn commit(self) -> crate::Result<()> {
        match self {
            BasicHandle::Pool(_) => Ok(()),
            BasicHandle::Tx(tx) => tx.commit().await.map_err(map_err),
            BasicHandle::Conn(_) => Ok(()),
        }
    }

    pub async fn rollback(self) -> crate::Result<()> {
        match self {
            BasicHandle::Pool(_) => Ok(()),
            BasicHandle::Tx(tx) => tx.rollback().await.map_err(map_err),
            BasicHandle::Conn(_) => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct ExecutorImpl<'h, 'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
{
    pub handle: &'h mut BasicHandle<'c, DB>,
}

pub trait AsExecutor {
    type Executor<'h>: sqlx::Executor<'h>
    where
        Self: 'h;

    fn as_executor<'h>(&'h mut self) -> Self::Executor<'h>;
}

impl<'c, DB> AsExecutor for BasicHandle<'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
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
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
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
            BasicHandle::Pool(pool) => pool.fetch_many(query),
            BasicHandle::Tx(tx) => tx.fetch_many(query),
            BasicHandle::Conn(conn) => conn.fetch_many(query),
        }
    }

    /// Execute the query and returns at most one row.
    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>>
    where
        'c: 'e,
        'h: 'e,
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        match self.handle {
            BasicHandle::Pool(pool) => pool.fetch_optional(query),
            BasicHandle::Tx(tx) => tx.fetch_optional(query),
            BasicHandle::Conn(conn) => conn.fetch_optional(query),
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
    ) -> BoxFuture<'e, Result<<Self::Database as sqlx::Database>::Statement<'q>, sqlx::Error>>
    where
        'c: 'e,
        'h: 'e,
    {
        match self.handle {
            BasicHandle::Pool(pool) => pool.prepare_with(sql, parameters),
            BasicHandle::Tx(tx) => tx.prepare_with(sql, parameters),
            BasicHandle::Conn(conn) => conn.prepare_with(sql, parameters),
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
            BasicHandle::Pool(pool) => pool.describe(sql),
            BasicHandle::Tx(tx) => tx.describe(sql),
            BasicHandle::Conn(conn) => conn.describe(sql),
        }
    }
}
