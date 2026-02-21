use futures_core::{future::BoxFuture, stream::BoxStream};

use sqlx::Acquire;

// impl From<sqlx::Error> for crate::Error {
//     fn from(value: sqlx::Error) -> Self {
//         Self::Database(anyhow::Error::new(value))
//     }
// }

// sealed trait + blanket impl
// not works
// mod sealed {
//     pub trait SqlxDB: sqlx::Database {}

//     impl<DB> SqlxDB for DB
//     where
//         DB: sqlx::Database,
//         for<'e> &'e mut DB::Connection:
//             sqlx::Executor<'e, Database = DB>,
//     {
//     }
// }

// pub trait DBFullConstraints: sqlx::Database
// where
//     for<'e> &'e mut Self::Connection: sqlx::Executor<'e, Database = Self>,
// {
// }

// impl DBFullConstraints for sqlx::Sqlite where
//     for<'e> &'e mut Self::Connection: sqlx::Executor<'e, Database = Self>
// {
// }
// impl DBFullConstraints for sqlx::Postgres where
//     for<'e> &'e mut Self::Connection: sqlx::Executor<'e, Database = Self>
// {
// }
// impl DBFullConstraints for sqlx::MySql where
//     for<'e> &'e mut Self::Connection: sqlx::Executor<'e, Database = Self>
// {
// }

// nightly
// #![feature(trait_alias)]
// pub trait SqlxDB = sqlx::Database
// where
//     for<'e> &'e mut <Self as sqlx::Database>::Connection:
//         sqlx::Executor<'e, Database = Self>;

// not works
// macro_rules! sqlx_db {
//     ($DB:ident) => {
//         $DB: sqlx::Database,
//         for<'e> &'e mut <$DB as sqlx::Database>::Connection:
//             sqlx::Executor<'e, Database = $DB>
//     };
// }

#[derive(Debug)]
pub enum Handle<'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
{
    Pool(sqlx::Pool<DB>),
    Transaction(sqlx::Transaction<'c, DB>),
    Connection(sqlx::pool::PoolConnection<DB>),
}

impl<'c, DB> Handle<'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
{
    pub async fn begin(&mut self) -> crate::Result<Handle<'_, DB>> {
        let tx = match self {
            Handle::Pool(pool) => pool.begin().await,
            Handle::Transaction(tx) => tx.begin().await,
            Handle::Connection(conn) => conn.begin().await,
        }
        .map_err(crate::db::map_err)?;
        Ok(Handle::Transaction(tx))
    }

    pub async fn commit(self) -> crate::Result<()> {
        match self {
            Handle::Pool(_) => Ok(()),
            Handle::Transaction(tx) => {
                tx.commit().await.map_err(crate::db::map_err)
            }
            Handle::Connection(_) => Ok(()),
        }
    }

    pub async fn rollback(self) -> crate::Result<()> {
        match self {
            Handle::Pool(_) => Ok(()),
            Handle::Transaction(tx) => {
                tx.rollback().await.map_err(crate::db::map_err)
            }
            Handle::Connection(_) => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct ExecutorImpl<'h, 'c, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
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
