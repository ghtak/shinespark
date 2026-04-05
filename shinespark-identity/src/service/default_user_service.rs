use std::sync::Arc;

use shinespark::crypto::password::PasswordService;

use crate::entity::{User, UserIdentity, UserWithIdentities, UserWithRoles};
use crate::repository::UserRepository;
use crate::service::user_service::{CreateUserCommand, UserService};
use crate::service::{FindUserQuery, InitialCredentials, UpdateUserCommand};

pub struct DefaultUserService<T: UserRepository + ?Sized, P: PasswordService> {
    pub user_repository: Arc<T>,
    pub password_service: Arc<P>,
}

impl<T: UserRepository + ?Sized, P: PasswordService> DefaultUserService<T, P> {
    pub fn new(user_repository: Arc<T>, password_service: Arc<P>) -> Self {
        Self {
            user_repository,
            password_service,
        }
    }
}

#[async_trait::async_trait]
impl<T: UserRepository + ?Sized, P: PasswordService> UserService for DefaultUserService<T, P> {
    async fn create_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreateUserCommand,
    ) -> shinespark::Result<UserWithIdentities> {
        let user = User::new(
            uuid::Uuid::new_v4(),
            command.name.clone(),
            command.email.clone(),
        );

        let user = self.user_repository.create_user(handle, user).await?;
        let (provider, provider_uid, credential_hash) = match command.credentials {
            InitialCredentials::Local { password } => (
                crate::entity::AuthProvider::Local,
                command.email.clone(),
                Some(self.password_service.hash_password(password.as_bytes())?),
            ),
            InitialCredentials::Social {
                provider,
                provider_uid,
            } => (provider, provider_uid, None),
        };

        let user_identity = UserIdentity::new(user.id, provider, provider_uid, credential_hash);
        let user_identity = self
            .user_repository
            .create_identity(handle, user_identity)
            .await?;
        Ok(UserWithIdentities {
            user,
            identities: vec![user_identity],
        })
    }

    async fn find_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        query: FindUserQuery,
    ) -> shinespark::Result<Option<UserWithRoles>> {
        let user = self.user_repository.find_user(handle, query).await?;
        Ok(user)
    }

    async fn update_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: UpdateUserCommand,
    ) -> shinespark::Result<User> {
        let user = self.user_repository.update_user(handle, command).await?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use shinespark::crypto::password::B64PasswordService;

    pub struct MockUserRepository {
        pub users: std::sync::Mutex<Vec<User>>,
        pub identities: std::sync::Mutex<Vec<UserIdentity>>,
    }

    impl MockUserRepository {
        pub fn new() -> Self {
            Self {
                users: std::sync::Mutex::new(Vec::new()),
                identities: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        async fn create_user(
            &self,
            _handle: &mut shinespark::db::Handle<'_>,
            mut user: super::User,
        ) -> shinespark::Result<super::User> {
            let mut users = self.users.lock().unwrap();
            let new_id = (users.len() as i64) + 1;
            user.id = new_id;
            users.push(user.clone());
            Ok(user)
        }

        async fn create_identity(
            &self,
            _handle: &mut shinespark::db::Handle<'_>,
            mut user_identity: super::UserIdentity,
        ) -> shinespark::Result<super::UserIdentity> {
            let mut identities = self.identities.lock().unwrap();
            let new_id = (identities.len() as i64) + 1;
            user_identity.id = new_id;
            identities.push(user_identity.clone());
            Ok(user_identity)
        }

        async fn find_user(
            &self,
            _handle: &mut shinespark::db::Handle<'_>,
            query: super::FindUserQuery,
        ) -> shinespark::Result<Option<super::UserWithRoles>> {
            let users = self.users.lock().unwrap();
            let user = users.iter().find(|u| {
                let id_match = query.id.map_or(true, |id| u.id == id);
                let uid_match = query.uid.map_or(true, |uid| u.uid == uid);
                let email_match = query.email.as_ref().map_or(true, |email| &u.email == email);
                id_match && uid_match && email_match
            });

            Ok(user.map(|u| super::UserWithRoles {
                user: u.clone(),
                role_ids: vec![],
            }))
        }

        async fn delete_user(
            &self,
            _handle: &mut shinespark::db::Handle<'_>,
            user_id: i64,
        ) -> shinespark::Result<User> {
            self.update_user(
                _handle,
                UpdateUserCommand {
                    id: user_id,
                    status: Some(crate::entity::UserStatus::Deleted),
                },
            )
            .await
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
    }

    use crate::{repository::DefaultUserRepository, service::InitialCredentials};

    use super::*;

    #[test]
    fn test_service() {
        let password_service = Arc::new(B64PasswordService {});
        let user_repository = Arc::new(MockUserRepository::new());
        let _service = DefaultUserService::new(user_repository, password_service);
    }

    async fn remove_test_user(
        database: &shinespark::db::Database,
        user_repository: Arc<dyn UserRepository>,
    ) {
        let exist_user = user_repository
            .find_user(
                &mut database.handle(),
                FindUserQuery::new().email("test".to_string()),
            )
            .await
            .unwrap();

        if let Some(u) = exist_user {
            user_repository
                .delete_user(&mut database.handle(), u.user.id)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_create_user() {
        let database = shinespark::db::Database::new_for_test().await.unwrap();

        let use_mock = false;
        let user_repository: Arc<dyn UserRepository> = if use_mock {
            Arc::new(MockUserRepository::new())
        } else {
            Arc::new(DefaultUserRepository {})
        };

        let service =
            DefaultUserService::new(user_repository.clone(), Arc::new(B64PasswordService {}));

        remove_test_user(&database, user_repository.clone()).await;

        let user = service
            .find_user(
                &mut database.handle(),
                FindUserQuery::new().email("test".to_string()),
            )
            .await
            .unwrap();

        if let Some(u) = user {
            assert_eq!(u.user.status, crate::entity::UserStatus::Deleted);
        }

        let mut tx = database.tx().await.unwrap();
        let command = CreateUserCommand {
            name: "test".to_string(),
            email: "test".to_string(),
            credentials: InitialCredentials::Local {
                password: "test".to_string(),
            },
        };

        let u = service.create_user(&mut tx, command).await.unwrap();

        let command = UpdateUserCommand {
            id: u.user.id,
            status: Some(crate::entity::UserStatus::Active),
        };
        let _updated_user = service.update_user(&mut tx, command).await;

        tx.commit().await.unwrap();

        let user = service
            .find_user(
                &mut database.handle(),
                FindUserQuery::new().email("test".to_string()),
            )
            .await
            .unwrap();

        assert!(user.is_some());
        assert_eq!(
            user.as_ref().unwrap().user.status,
            crate::entity::UserStatus::Active
        );
    }
}
