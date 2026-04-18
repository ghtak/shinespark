use crate::entities::{Permission, Role};

#[async_trait::async_trait]
pub trait RbacRepository: Send + Sync + 'static {
    // ── Permission CRUD ──────────────────────────────────────────────────────
    async fn create_permission(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        code: &str,
        description: &str,
    ) -> shinespark::Result<Permission>;

    async fn delete_permission(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        id: i64,
    ) -> shinespark::Result<()>;

    async fn list_permissions(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<Permission>>;

    async fn find_permission_by_code(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        code: &str,
    ) -> shinespark::Result<Option<Permission>>;

    /// `shs_iam_role_permission` 에서 permission_id 기준 전체 삭제 (delete_permission cascade 용)
    async fn delete_role_permissions_by_permission_id(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        permission_id: i64,
    ) -> shinespark::Result<()>;

    // ── Role CRUD ────────────────────────────────────────────────────────────
    async fn create_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        name: &str,
        description: &str,
    ) -> shinespark::Result<Role>;

    async fn delete_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        id: i64,
    ) -> shinespark::Result<()>;

    async fn list_roles(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<Role>>;

    /// `shs_iam_role_permission` 에서 role_id 기준 전체 삭제 (delete_role cascade 용)
    async fn delete_role_permissions_by_role_id(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
    ) -> shinespark::Result<()>;

    /// `shs_iam_user_role` 에서 role_id 기준 전체 삭제 (delete_role cascade 용)
    async fn delete_user_roles_by_role_id(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
    ) -> shinespark::Result<()>;

    // ── 기존 메서드 ───────────────────────────────────────────────────────────
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
