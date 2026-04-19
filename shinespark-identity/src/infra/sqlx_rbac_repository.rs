use shinespark::db::SqlStatement;

use crate::{
    entities::{Permission, Role},
    repositories::RbacRepository,
};

enum RbacQuery {
    LoadRolePermissions,
}

impl SqlStatement for RbacQuery {
    fn as_str(&self) -> &'static str {
        match self {
            RbacQuery::LoadRolePermissions => {
                include_str!("../../sql/rbac_repository/load_role_permissions.sql")
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
        "SELECT * FROM shs_iam_role WHERE name = $1"
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
        sqlx::query(
            "INSERT INTO shs_iam_user_role (user_id, role_id) VALUES ($1, $2) ON CONFLICT (user_id, role_id) DO NOTHING",
        )
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
        "DELETE FROM shs_iam_user_role WHERE user_id = $1 AND role_id = $2"
            .as_query()
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
        r#"
        INSERT INTO
            shs_iam_role_permission (role_id, permission_id)
        VALUES ($1, $2)
        ON CONFLICT (role_id, permission_id) DO NOTHING
        "#
        .as_query()
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
        "DELETE FROM shs_iam_role_permission WHERE role_id = $1 AND permission_id = $2"
            .as_query()
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
        "INSERT INTO shs_iam_permission (code, description) VALUES ($1, $2) RETURNING *"
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
        "DELETE FROM shs_iam_permission WHERE id = $1"
            .as_query()
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
        "SELECT * FROM shs_iam_permission ORDER BY code"
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
        "SELECT * FROM shs_iam_permission WHERE code = $1"
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
        "DELETE FROM shs_iam_role_permission WHERE permission_id = $1"
            .as_query()
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
        "INSERT INTO shs_iam_role (name, description) VALUES ($1, $2) RETURNING *"
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
        "DELETE FROM shs_iam_role WHERE id = $1"
            .as_query()
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
        "SELECT * FROM shs_iam_role ORDER BY name"
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
        "DELETE FROM shs_iam_role_permission WHERE role_id = $1"
            .as_query()
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
        "DELETE FROM shs_iam_user_role WHERE role_id = $1"
            .as_query()
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

        repo.assign_role_to_user(&mut handle, user_id, role.id).await.unwrap();

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
