mod handle;

use handle::*;

#[cfg(not(any(
    feature = "db-driver-postgres",
    feature = "db-driver-sqlite",
    feature = "db-driver-mysql"
)))]
compile_error!(
    "반드시 하나 이상의 데이터베이스 피처(db-driver-postgres, db-driver-sqlite, db-driver-mysql)를 선택해야 합니다."
);

#[cfg(all(feature = "db-driver-postgres", feature = "db-driver-sqlite"))]
compile_error!("여러 개의 데이터베이스 피처를 동시에 활성화할 수 없습니다.");

#[cfg(all(feature = "db-driver-postgres", feature = "db-driver-mysql"))]
compile_error!("여러 개의 데이터베이스 피처를 동시에 활성화할 수 없습니다.");

#[cfg(all(feature = "db-driver-sqlite", feature = "db-driver-mysql"))]
compile_error!("여러 개의 데이터베이스 피처를 동시에 활성화할 수 없습니다.");

#[cfg(feature = "db-driver-postgres")]
pub type Driver = sqlx::Postgres;

#[cfg(feature = "db-driver-sqlite")]
pub type Driver = sqlx::Sqlite;

#[cfg(feature = "db-driver-mysql")]
pub type Driver = sqlx::MySql;

pub type PostgresHandle<'c> = BasicHandle<'c, sqlx::Postgres>;
pub type SqliteHandle<'c> = BasicHandle<'c, sqlx::Sqlite>;
pub type MySqlHandle<'c> = BasicHandle<'c, sqlx::MySql>;

pub type Handle<'c> = BasicHandle<'c, Driver>;

#[derive(Debug, Clone)]
pub struct Database {
    pub inner: sqlx::Pool<Driver>,
}

impl Database {
    pub async fn new(config: &crate::config::DatabaseConfig) -> crate::Result<Self> {
        let inner = sqlx::pool::PoolOptions::<Driver>::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await
            .map_err(|e| crate::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(Self { inner })
    }

    pub fn handle(&self) -> Handle<'_> {
        Handle::Pool(self.inner.clone())
    }

    pub async fn tx(&self) -> crate::Result<Handle<'_>> {
        let tx = self.inner.begin().await.map_err(map_err)?;
        Ok(Handle::Tx(tx))
    }

    pub async fn conn(&self) -> crate::Result<Handle<'_>> {
        let conn = self.inner.acquire().await.map_err(map_err)?;
        Ok(Handle::Conn(conn))
    }

    pub async fn new_for_test() -> crate::Result<Self> {
        use std::env;
        dotenvy::dotenv().ok();
        let config = crate::config::DatabaseConfig {
            url: env::var("DATABASE_URL").unwrap(),
            max_connections: 1,
        };
        Self::new(&config).await
    }
}

pub fn bind_opt<'q, T>(
    query_builder: &mut sqlx::QueryBuilder<'q, Driver>,
    sql: &str,
    value: &'q Option<T>,
) where
    T: sqlx::Type<Driver> + sqlx::Encode<'q, Driver> + Send + 'q,
{
    if let Some(value) = value {
        query_builder.push(sql).push_bind(value);
    }
}

pub trait QueryFilter {
    fn apply<'q>(&'q self, query_builder: &mut sqlx::QueryBuilder<'q, Driver>)
    -> crate::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_new_database() {
        let database = Database::new_for_test().await.unwrap();
        {
            let mut h = database.handle();
            sqlx::query("SELECT 1").execute(h.inner()).await.unwrap();
        }

        {
            let mut tx = database.tx().await.unwrap();
            sqlx::query("SELECT 1").execute(tx.inner()).await.unwrap();
            tx.commit().await.unwrap();
        }

        {
            let mut c = database.conn().await.unwrap();
            sqlx::query("SELECT 1").execute(c.inner()).await.unwrap();
        }
    }
}
