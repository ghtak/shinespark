use shinespark::db::AppDbHandle;

#[async_trait::async_trait]
pub trait UserRepository: Sync + Send {
    async fn find_user(
        &self,
        h: &mut AppDbHandle<'_>,
    ) -> shinespark::Result<Vec<crate::entity::User>>;
}
