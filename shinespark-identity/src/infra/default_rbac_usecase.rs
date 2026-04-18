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
