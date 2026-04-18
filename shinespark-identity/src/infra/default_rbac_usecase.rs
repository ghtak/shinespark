use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use crate::{repositories::RbacRepository, usecases::RbacUsecase};

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
