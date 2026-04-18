use shinespark::db::SqlStatement;

use crate::{
    entities::{Permission, Role},
    repositories::RbacRepository,
};

enum RbacQuery {
    // 기존
    LoadRolePermissions,
    FindRoleByName,
    AssignRoleToUser,
    RemoveRoleFromUser,
    AddPermissionToRole,
    RemovePermissionFromRole,
    // Permission CRUD
    CreatePermission,
    DeletePermission,
    ListPermissions,
    FindPermissionByCode,
    DeleteRolePermissionsByPermissionId,
    // Role CRUD
    CreateRole,
    DeleteRole,
    ListRoles,
    DeleteRolePermissionsByRoleId,
    DeleteUserRolesByRoleId,
}

impl SqlStatement for RbacQuery {
    fn as_str(&self) -> &'static str {
        match self {
            RbacQuery::LoadRolePermissions => {
                include_str!("../../sql/rbac_repository/load_role_permissions.sql")
            }
            RbacQuery::FindRoleByName => {
                include_str!("../../sql/rbac_repository/find_role_by_name.sql")
            }
            RbacQuery::AssignRoleToUser => {
                include_str!("../../sql/rbac_repository/assign_role_to_user.sql")
            }
            RbacQuery::RemoveRoleFromUser => {
                include_str!("../../sql/rbac_repository/remove_role_from_user.sql")
            }
            RbacQuery::AddPermissionToRole => {
                include_str!("../../sql/rbac_repository/add_permission_to_role.sql")
            }
            RbacQuery::RemovePermissionFromRole => {
                include_str!("../../sql/rbac_repository/remove_permission_from_role.sql")
            }
            RbacQuery::CreatePermission => {
                include_str!("../../sql/rbac_repository/create_permission.sql")
            }
            RbacQuery::DeletePermission => {
                include_str!("../../sql/rbac_repository/delete_permission.sql")
            }
            RbacQuery::ListPermissions => {
                include_str!("../../sql/rbac_repository/list_permissions.sql")
            }
            RbacQuery::FindPermissionByCode => {
                include_str!("../../sql/rbac_repository/find_permission_by_code.sql")
            }
            RbacQuery::DeleteRolePermissionsByPermissionId => {
                include_str!("../../sql/rbac_repository/delete_role_permissions_by_permission_id.sql")
            }
            RbacQuery::CreateRole => {
                include_str!("../../sql/rbac_repository/create_role.sql")
            }
            RbacQuery::DeleteRole => {
                include_str!("../../sql/rbac_repository/delete_role.sql")
            }
            RbacQuery::ListRoles => {
                include_str!("../../sql/rbac_repository/list_roles.sql")
            }
            RbacQuery::DeleteRolePermissionsByRoleId => {
                include_str!("../../sql/rbac_repository/delete_role_permissions_by_role_id.sql")
            }
            RbacQuery::DeleteUserRolesByRoleId => {
                include_str!("../../sql/rbac_repository/delete_user_roles_by_role_id.sql")
            }
        }
    }
}

pub struct SqlxRbacRepository {}

impl SqlxRbacRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl RbacRepository for SqlxRbacRepository {
    async fn load_role_permissions(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<(i64, String)>> {
        RbacQuery::LoadRolePermissions
            .as_query_as::<(i64, String)>()
            .fetch_all(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))
    }

    async fn find_role_by_name(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        name: &str,
    ) -> shinespark::Result<Option<Role>> {
        RbacQuery::FindRoleByName
            .as_query_as::<Role>()
            .bind(name)
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))
    }

    async fn assign_role_to_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
        role_id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query(RbacQuery::AssignRoleToUser.as_str())
            .bind(user_id)
            .bind(role_id)
            .execute(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn remove_role_from_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
        role_id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query(RbacQuery::RemoveRoleFromUser.as_str())
            .bind(user_id)
            .bind(role_id)
            .execute(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn add_permission_to_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query(RbacQuery::AddPermissionToRole.as_str())
            .bind(role_id)
            .bind(permission_id)
            .execute(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn remove_permission_from_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query(RbacQuery::RemovePermissionFromRole.as_str())
            .bind(role_id)
            .bind(permission_id)
            .execute(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(())
    }

    // ── Permission CRUD ──────────────────────────────────────────────────────

    async fn create_permission(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        code: &str,
        description: &str,
    ) -> shinespark::Result<Permission> {
        RbacQuery::CreatePermission
            .as_query_as::<Permission>()
            .bind(code)
            .bind(description)
            .fetch_one(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))
    }

    async fn delete_permission(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query(RbacQuery::DeletePermission.as_str())
            .bind(id)
            .execute(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn list_permissions(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<Permission>> {
        RbacQuery::ListPermissions
            .as_query_as::<Permission>()
            .fetch_all(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))
    }

    async fn find_permission_by_code(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        code: &str,
    ) -> shinespark::Result<Option<Permission>> {
        RbacQuery::FindPermissionByCode
            .as_query_as::<Permission>()
            .bind(code)
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))
    }

    async fn delete_role_permissions_by_permission_id(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        permission_id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query(RbacQuery::DeleteRolePermissionsByPermissionId.as_str())
            .bind(permission_id)
            .execute(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(())
    }

    // ── Role CRUD ────────────────────────────────────────────────────────────

    async fn create_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        name: &str,
        description: &str,
    ) -> shinespark::Result<Role> {
        RbacQuery::CreateRole
            .as_query_as::<Role>()
            .bind(name)
            .bind(description)
            .fetch_one(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))
    }

    async fn delete_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query(RbacQuery::DeleteRole.as_str())
            .bind(id)
            .execute(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn list_roles(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<Role>> {
        RbacQuery::ListRoles
            .as_query_as::<Role>()
            .fetch_all(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))
    }

    async fn delete_role_permissions_by_role_id(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query(RbacQuery::DeleteRolePermissionsByRoleId.as_str())
            .bind(role_id)
            .execute(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn delete_user_roles_by_role_id(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
    ) -> shinespark::Result<()> {
        sqlx::query(RbacQuery::DeleteUserRolesByRoleId.as_str())
            .bind(role_id)
            .execute(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::RbacRepository;

    #[tokio::test]
    #[ignore]
    async fn test_load_role_permissions() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let repo = SqlxRbacRepository::new();
        let mut handle = db.handle();

        let pairs = repo.load_role_permissions(&mut handle).await.unwrap();
        assert!(!pairs.is_empty());
        assert!(pairs.iter().any(|(_, code)| code == "*.*.all"));
        assert!(pairs.iter().any(|(_, code)| code == "user.read.own"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_role_by_name() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let repo = SqlxRbacRepository::new();
        let mut handle = db.handle();

        let admin = repo.find_role_by_name(&mut handle, "admin").await.unwrap();
        assert!(admin.is_some());
        assert_eq!(admin.unwrap().name, "admin");

        let user = repo.find_role_by_name(&mut handle, "user").await.unwrap();
        assert!(user.is_some());

        let none = repo.find_role_by_name(&mut handle, "nonexistent").await.unwrap();
        assert!(none.is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_assign_and_remove_role_to_user() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let repo = SqlxRbacRepository::new();
        let mut handle = db.tx().await.unwrap();

        // 테스트용 임시 유저 삽입 (트랜잭션 내 — rollback으로 정리)
        let test_uid = uuid::Uuid::new_v4();
        let (user_id,): (i64,) = sqlx::query_as(
            "INSERT INTO shs_iam_user (uid, name, email, status) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(test_uid)
        .bind("test_rbac_user")
        .bind(format!("test_rbac_{}@example.com", test_uid))
        .bind("active")
        .fetch_one(handle.inner())
        .await
        .unwrap();

        let role = repo.find_role_by_name(&mut handle, "user").await.unwrap().unwrap();

        // 역할 부여
        repo.assign_role_to_user(&mut handle, user_id, role.id).await.unwrap();

        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM shs_iam_user_role WHERE user_id = $1 AND role_id = $2",
        )
        .bind(user_id)
        .bind(role.id)
        .fetch_one(handle.inner())
        .await
        .unwrap();
        assert_eq!(count, 1);

        // 중복 부여는 에러 없이 멱등
        repo.assign_role_to_user(&mut handle, user_id, role.id).await.unwrap();

        // 역할 제거
        repo.remove_role_from_user(&mut handle, user_id, role.id).await.unwrap();

        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM shs_iam_user_role WHERE user_id = $1 AND role_id = $2",
        )
        .bind(user_id)
        .bind(role.id)
        .fetch_one(handle.inner())
        .await
        .unwrap();
        assert_eq!(count, 0);

        handle.rollback().await.unwrap();
    }
}
