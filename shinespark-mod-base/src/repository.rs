use crate::entity::SimpleEntity;

#[async_trait::async_trait]
pub trait SimpleRepository<DB>:
    shinespark::db::BaseRepository<SimpleEntity, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
{
}
