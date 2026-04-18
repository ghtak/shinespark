use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use crate::{
    entities::{Permission, Role},
    repositories::RbacRepository,
    usecases::{CreatePermissionCommand, CreateRoleCommand, RbacUsecase},
};

pub struct DefaultRbacUsecase<R: RbacRepository> {
    repository: Arc<R>,
    cache: Arc<RwLock<HashMap<i64, HashSet<String>>>>,
}

impl<R: RbacRepository> DefaultRbacUsecase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl<R: RbacRepository> RbacUsecase for DefaultRbacUsecase<R> {
    async fn load(&self, handle: &mut shinespark::db::Handle<'_>) -> shinespark::Result<()> {
        let pairs = self.repository.load_role_permissions(handle).await?;
        let mut map: HashMap<i64, HashSet<String>> = HashMap::new();
        for (role_id, code) in pairs {
            map.entry(role_id).or_default().insert(code);
        }
        *self.cache.write().unwrap() = map;
        Ok(())
    }

    fn check_perm(&self, role_ids: &[i64], permission: &str) -> bool {
        let cache = self.cache.read().unwrap();
        role_ids.iter().any(|role_id| {
            cache
                .get(role_id)
                .map_or(false, |perms| perms.contains("*.*.all") || perms.contains(permission))
        })
    }

    // ── Permission CRUD ──────────────────────────────────────────────────────

    async fn create_permission(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreatePermissionCommand,
    ) -> shinespark::Result<Permission> {
        if self.repository.find_permission_by_code(handle, &command.code).await?.is_some() {
            return Err(shinespark::Error::AlreadyExists);
        }
        self.repository.create_permission(handle, &command.code, &command.description).await
    }

    async fn delete_permission(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        permission_id: i64,
    ) -> shinespark::Result<()> {
        self.repository.delete_role_permissions_by_permission_id(handle, permission_id).await?;
        self.repository.delete_permission(handle, permission_id).await?;
        self.load(handle).await
    }

    async fn list_permissions(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<Permission>> {
        self.repository.list_permissions(handle).await
    }

    async fn find_permission_by_code(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        code: &str,
    ) -> shinespark::Result<Option<Permission>> {
        self.repository.find_permission_by_code(handle, code).await
    }

    // ── Role CRUD ────────────────────────────────────────────────────────────

    async fn create_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreateRoleCommand,
    ) -> shinespark::Result<Role> {
        if self.repository.find_role_by_name(handle, &command.name).await?.is_some() {
            return Err(shinespark::Error::AlreadyExists);
        }
        self.repository.create_role(handle, &command.name, &command.description).await
    }

    async fn delete_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
    ) -> shinespark::Result<()> {
        self.repository.delete_user_roles_by_role_id(handle, role_id).await?;
        self.repository.delete_role_permissions_by_role_id(handle, role_id).await?;
        self.repository.delete_role(handle, role_id).await?;
        self.load(handle).await
    }

    async fn list_roles(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<Role>> {
        self.repository.list_roles(handle).await
    }

    // ── 역할-권한 링크 (code/name 기반 상위 레벨) ────────────────────────────

    async fn assign_permission_to_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_name: &str,
        permission_code: &str,
    ) -> shinespark::Result<()> {
        let role = self
            .repository
            .find_role_by_name(handle, role_name)
            .await?
            .ok_or(shinespark::Error::NotFound)?;
        let perm = self
            .repository
            .find_permission_by_code(handle, permission_code)
            .await?
            .ok_or(shinespark::Error::NotFound)?;
        self.add_permission_to_role(handle, role.id, perm.id).await
    }

    async fn revoke_permission_from_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_name: &str,
        permission_code: &str,
    ) -> shinespark::Result<()> {
        let role = self
            .repository
            .find_role_by_name(handle, role_name)
            .await?
            .ok_or(shinespark::Error::NotFound)?;
        let perm = self
            .repository
            .find_permission_by_code(handle, permission_code)
            .await?
            .ok_or(shinespark::Error::NotFound)?;
        self.remove_permission_from_role(handle, role.id, perm.id).await
    }

    // ── 유저-역할 관리 ────────────────────────────────────────────────────────

    async fn assign_role_to_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
        role_name: &str,
    ) -> shinespark::Result<()> {
        let role = self
            .repository
            .find_role_by_name(handle, role_name)
            .await?
            .ok_or(shinespark::Error::NotFound)?;
        self.repository.assign_role_to_user(handle, user_id, role.id).await
    }

    async fn add_permission_to_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()> {
        self.repository.add_permission_to_role(handle, role_id, permission_id).await?;
        self.load(handle).await
    }

    async fn remove_permission_from_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()> {
        self.repository.remove_permission_from_role(handle, role_id, permission_id).await?;
        self.load(handle).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{infra::SqlxRbacRepository, repositories::RbacRepository, usecases::RbacUsecase};

    fn make_usecase() -> DefaultRbacUsecase<SqlxRbacRepository> {
        DefaultRbacUsecase::new(Arc::new(SqlxRbacRepository::new()))
    }

    #[test]
    fn test_check_perm_before_load_returns_false() {
        let usecase = make_usecase();
        assert!(!usecase.check_perm(&[1, 2], "user.read.own"));
        assert!(!usecase.check_perm(&[], "*.*.all"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_load_and_check_perm() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let usecase = make_usecase();
        let mut handle = db.handle();
        usecase.load(&mut handle).await.unwrap();

        let repo = SqlxRbacRepository::new();
        let admin = repo.find_role_by_name(&mut handle, "admin").await.unwrap().unwrap();
        let user = repo.find_role_by_name(&mut handle, "user").await.unwrap().unwrap();

        // admin: *.*.all 로 모든 권한 통과
        assert!(usecase.check_perm(&[admin.id], "user.read.all"));
        assert!(usecase.check_perm(&[admin.id], "user.create.all"));
        assert!(usecase.check_perm(&[admin.id], "anything.random"));

        // user: own 범위 권한만 보유
        assert!(usecase.check_perm(&[user.id], "user.read.own"));
        assert!(usecase.check_perm(&[user.id], "user.update.own"));
        assert!(usecase.check_perm(&[user.id], "user.delete.own"));

        // user: all 범위 권한 없음
        assert!(!usecase.check_perm(&[user.id], "user.create.all"));
        assert!(!usecase.check_perm(&[user.id], "user.read.all"));

        // 멀티 역할: 하나라도 매칭되면 true
        assert!(usecase.check_perm(&[admin.id, user.id], "user.read.all"));

        // 역할 없음 -> false
        assert!(!usecase.check_perm(&[], "user.read.own"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_assign_role_to_user_role_not_found() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let usecase = make_usecase();
        let mut handle = db.handle();

        let result = usecase.assign_role_to_user(&mut handle, 1, "nonexistent_role").await;
        assert!(matches!(result, Err(shinespark::Error::NotFound)));
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_and_delete_permission() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let usecase = make_usecase();
        let mut handle = db.tx().await.unwrap();

        let perm = usecase
            .create_permission(
                &mut handle,
                crate::usecases::CreatePermissionCommand {
                    code: "test.crud.own".to_string(),
                    description: "테스트용 권한".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(perm.code, "test.crud.own");

        // 중복 code → AlreadyExists
        let dup = usecase
            .create_permission(
                &mut handle,
                crate::usecases::CreatePermissionCommand {
                    code: "test.crud.own".to_string(),
                    description: "중복".to_string(),
                },
            )
            .await;
        assert!(matches!(dup, Err(shinespark::Error::AlreadyExists)));

        // 삭제
        usecase.delete_permission(&mut handle, perm.id).await.unwrap();
        let found = usecase.find_permission_by_code(&mut handle, "test.crud.own").await.unwrap();
        assert!(found.is_none());

        handle.rollback().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_and_delete_role() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let usecase = make_usecase();
        let mut handle = db.tx().await.unwrap();

        let role = usecase
            .create_role(
                &mut handle,
                crate::usecases::CreateRoleCommand {
                    name: "test_role".to_string(),
                    description: "테스트 역할".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(role.name, "test_role");

        // 중복 name → AlreadyExists
        let dup = usecase
            .create_role(
                &mut handle,
                crate::usecases::CreateRoleCommand {
                    name: "test_role".to_string(),
                    description: "중복".to_string(),
                },
            )
            .await;
        assert!(matches!(dup, Err(shinespark::Error::AlreadyExists)));

        // 삭제
        usecase.delete_role(&mut handle, role.id).await.unwrap();
        let roles = usecase.list_roles(&mut handle).await.unwrap();
        assert!(!roles.iter().any(|r| r.name == "test_role"));

        handle.rollback().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_permission_cascades_role_permission() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let usecase = make_usecase();
        let mut handle = db.tx().await.unwrap();
        usecase.load(&mut handle).await.unwrap();

        // 새 권한 생성 후 user 역할에 연결
        let perm = usecase
            .create_permission(
                &mut handle,
                crate::usecases::CreatePermissionCommand {
                    code: "test.cascade.all".to_string(),
                    description: "cascade 테스트".to_string(),
                },
            )
            .await
            .unwrap();

        let repo = SqlxRbacRepository::new();
        let user_role = repo.find_role_by_name(&mut handle, "user").await.unwrap().unwrap();
        usecase.add_permission_to_role(&mut handle, user_role.id, perm.id).await.unwrap();
        assert!(usecase.check_perm(&[user_role.id], "test.cascade.all"));

        // permission 삭제 → role_permission도 삭제 → 캐시 갱신
        usecase.delete_permission(&mut handle, perm.id).await.unwrap();
        assert!(!usecase.check_perm(&[user_role.id], "test.cascade.all"));

        handle.rollback().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_assign_and_revoke_permission_by_name_code() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let usecase = make_usecase();
        let mut handle = db.tx().await.unwrap();
        usecase.load(&mut handle).await.unwrap();

        let repo = SqlxRbacRepository::new();
        let user_role = repo.find_role_by_name(&mut handle, "user").await.unwrap().unwrap();

        // user 역할에 없는 user.create.all 을 name/code 기반으로 부여
        assert!(!usecase.check_perm(&[user_role.id], "user.create.all"));
        usecase.assign_permission_to_role(&mut handle, "user", "user.create.all").await.unwrap();
        assert!(usecase.check_perm(&[user_role.id], "user.create.all"));

        // 해제
        usecase.revoke_permission_from_role(&mut handle, "user", "user.create.all").await.unwrap();
        assert!(!usecase.check_perm(&[user_role.id], "user.create.all"));

        // 존재하지 않는 role/permission → NotFound
        let err = usecase.assign_permission_to_role(&mut handle, "ghost", "user.read.own").await;
        assert!(matches!(err, Err(shinespark::Error::NotFound)));

        handle.rollback().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_permissions_and_roles() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let usecase = make_usecase();
        let mut handle = db.handle();

        let perms = usecase.list_permissions(&mut handle).await.unwrap();
        assert!(perms.iter().any(|p| p.code == "*.*.all"));

        let roles = usecase.list_roles(&mut handle).await.unwrap();
        assert!(roles.iter().any(|r| r.name == "admin"));
        assert!(roles.iter().any(|r| r.name == "user"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_add_and_remove_permission_refreshes_cache() {
        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let usecase = make_usecase();
        let mut handle = db.tx().await.unwrap();
        usecase.load(&mut handle).await.unwrap();

        let repo = SqlxRbacRepository::new();
        let user_role = repo.find_role_by_name(&mut handle, "user").await.unwrap().unwrap();

        // user 역할에 없는 권한: user.create.all
        let (perm_id,): (i64,) =
            sqlx::query_as("SELECT id FROM shs_iam_permission WHERE code = $1")
                .bind("user.create.all")
                .fetch_one(handle.inner())
                .await
                .unwrap();

        assert!(!usecase.check_perm(&[user_role.id], "user.create.all"));

        // 권한 추가 → 캐시 자동 갱신
        usecase.add_permission_to_role(&mut handle, user_role.id, perm_id).await.unwrap();
        assert!(usecase.check_perm(&[user_role.id], "user.create.all"));

        // 권한 제거 → 캐시 자동 갱신
        usecase.remove_permission_from_role(&mut handle, user_role.id, perm_id).await.unwrap();
        assert!(!usecase.check_perm(&[user_role.id], "user.create.all"));

        handle.rollback().await.unwrap();
    }
}
