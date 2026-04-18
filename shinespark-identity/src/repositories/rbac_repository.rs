use crate::entities::Role;

#[async_trait::async_trait]
pub trait RbacRepository: Send + Sync + 'static {
    async fn load_role_permissions(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<(i64, String)>>;

    async fn find_role_by_name(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        name: &str,
    ) -> shinespark::Result<Option<Role>>;

    async fn assign_role_to_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
        role_id: i64,
    ) -> shinespark::Result<()>;

    async fn remove_role_from_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
        role_id: i64,
    ) -> shinespark::Result<()>;

    async fn add_permission_to_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()>;

    async fn remove_permission_from_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()>;
}
