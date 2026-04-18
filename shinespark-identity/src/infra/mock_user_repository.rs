use crate::entities::{AuthProvider, User, UserAggregate, UserIdentity};
use crate::repositories::UserRepository;
use crate::usecases::{FindUserQuery, UpdateUserCommand};
use std::sync::Mutex;

pub struct MockUserRepository {
    pub users: Mutex<Vec<User>>,
    pub identities: Mutex<Vec<UserIdentity>>,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(Vec::new()),
            identities: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl UserRepository for MockUserRepository {
    async fn create_user(
        &self,
        _handle: &mut shinespark::db::Handle<'_>,
        mut user: User,
    ) -> shinespark::Result<User> {
        let mut users = self.users.lock().unwrap();
        let new_id = (users.len() as i64) + 1;
        user.id = new_id;
        users.push(user.clone());
        Ok(user)
    }

    async fn create_identity(
        &self,
        _handle: &mut shinespark::db::Handle<'_>,
        mut user_identity: UserIdentity,
    ) -> shinespark::Result<UserIdentity> {
        let mut identities = self.identities.lock().unwrap();
        let new_id = (identities.len() as i64) + 1;
        user_identity.id = new_id;
        identities.push(user_identity.clone());
        Ok(user_identity)
    }

    async fn find_user(
        &self,
        _handle: &mut shinespark::db::Handle<'_>,
        query: FindUserQuery,
    ) -> shinespark::Result<Option<UserAggregate>> {
        let users = self.users.lock().unwrap();
        let user = users.iter().find(|u| {
            let id_match = query.id.map_or(true, |id| u.id == id);
            let uid_match = query.uid.map_or(true, |uid| u.uid == uid);
            let email_match = query.email.as_ref().map_or(true, |email| &u.email == email);
            id_match && uid_match && email_match
        });

        Ok(user.map(|u| UserAggregate {
            user: u.clone(),
            role_ids: vec![],
            identities: vec![],
        }))
    }

    async fn update_user(
        &self,
        _handle: &mut shinespark::db::Handle<'_>,
        command: UpdateUserCommand,
    ) -> shinespark::Result<User> {
        let mut users = self.users.lock().unwrap();
        if let Some(user) = users.iter_mut().find(|u| u.id == command.id) {
            if let Some(status) = command.status {
                user.status = status;
            }
            user.updated_at = chrono::Utc::now();
            Ok(user.clone())
        } else {
            Err(shinespark::Error::NotFound)
        }
    }

    async fn find_user_by_identity(
        &self,
        _handle: &mut shinespark::db::Handle<'_>,
        _provider: AuthProvider,
        _provider_uid: String,
    ) -> shinespark::Result<Option<UserAggregate>> {
        let identity_opt = {
            let identities = self.identities.lock().unwrap();
            identities.iter().find(|i| i.provider == _provider && i.provider_uid == _provider_uid).cloned()
        };
        if let Some(identity) = identity_opt {
            let user_opt = self.users.lock().unwrap().iter().find(|u| u.id == identity.user_id).cloned();
            let user_identities = self.identities.lock().unwrap().iter().filter(|i| i.user_id == identity.user_id).cloned().collect();
            Ok(user_opt.map(|u| UserAggregate { user: u, role_ids: vec![], identities: user_identities }))
        } else {
            Ok(None)
        }
    }
}
