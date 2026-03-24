mod handle;

use handle::*;

#[cfg(not(any(feature = "postgres", feature = "sqlite", feature = "mysql")))]
compile_error!("반드시 하나 이상의 데이터베이스 피처(postgres, sqlite, mysql)를 선택해야 합니다.");

#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("여러 개의 데이터베이스 피처를 동시에 활성화할 수 없습니다.");

#[cfg(all(feature = "postgres", feature = "mysql"))]
compile_error!("여러 개의 데이터베이스 피처를 동시에 활성화할 수 없습니다.");

#[cfg(all(feature = "sqlite", feature = "mysql"))]
compile_error!("여러 개의 데이터베이스 피처를 동시에 활성화할 수 없습니다.");

#[cfg(feature = "postgres")]
pub type Driver = sqlx::Postgres;

#[cfg(feature = "sqlite")]
pub type Driver = sqlx::Sqlite;

#[cfg(feature = "mysql")]
pub type Driver = sqlx::MySql;

pub type Handle<'c> = BasicHandle<'c, Driver>;

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    //async fn run_generic_handle_test<DB>(pool: sqlx::Pool<DB>, dummy_query: &str)
    // where
    //     DB: sqlx::Database,
    //     for<'e> &'e mut <DB as sqlx::Database>::Connection: sqlx::Executor<'e, Database = DB>,
    //     for<'q> <DB as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    async fn run_handle_test(pool: sqlx::Pool<Driver>, dummy_query: &str) {
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
            let mut tx_handle = pool_handle
                .begin()
                .await
                .expect("Failed to begin transaction");

            {
                let exec = tx_handle.as_executor();
                sqlx::query(dummy_query)
                    .execute(exec)
                    .await
                    .expect("Failed to execute query via Transaction handle");
            }

            tx_handle
                .commit()
                .await
                .expect("Failed to commit transaction");
        }

        // 3. Test Connection handle
        {
            let conn = pool.acquire().await.expect("Failed to acquire connection");
            let mut handle = Handle::Conn(conn);
            let exec = handle.as_executor();
            sqlx::query(dummy_query)
                .execute(exec)
                .await
                .expect("Failed to execute query via Connection handle");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_basic_handle() {
        dotenvy::dotenv().ok();
        let pool = sqlx::Pool::<Driver>::connect(&env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();
        run_handle_test(pool, "SELECT 1").await;
    }
}
