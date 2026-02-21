#[async_trait::async_trait]
pub trait BaseRepository<T, DB>
where
    DB: sqlx::Database,
    for<'e> &'e mut DB::Connection: sqlx::Executor<'e, Database = DB>,
{
    async fn create(
        &self,
        h: &mut crate::db::Handle<'_, DB>,
        entity: T,
    ) -> crate::Result<T>;

    async fn find_by_id(
        &self,
        h: &mut crate::db::Handle<'_, DB>,
        id: i64,
    ) -> crate::Result<Option<T>>;

    async fn find_all(
        &self,
        h: &mut crate::db::Handle<'_, DB>,
    ) -> crate::Result<Vec<T>>;

    async fn update(
        &self,
        h: &mut crate::db::Handle<'_, DB>,
        entity: T,
    ) -> crate::Result<T>;

    async fn delete(
        &self,
        h: &mut crate::db::Handle<'_, DB>,
        id: i64,
    ) -> crate::Result<()>;
}
