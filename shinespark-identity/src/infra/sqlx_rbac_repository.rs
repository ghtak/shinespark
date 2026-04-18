use shinespark::db::SqlStatement;

use crate::{entities::Role, repositories::RbacRepository};

enum RbacQuery {
    LoadRolePermissions,
    FindRoleByName,
    AssignRoleToUser,
    RemoveRoleFromUser,
    AddPermissionToRole,
    RemovePermissionFromRole,
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
}
