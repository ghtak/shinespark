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
