use std::sync::Arc;

use crate::entity::{User, UserIdentity, UserWithIdentities, UserWithRoles};
use crate::repository::UserRepository;
use crate::service::user_service::{CreateUserCommand, UserService};
use crate::service::{FindUserQuery, InitialCredentials, UpdateUserCommand};

pub struct DefaultUserService<T: UserRepository + ?Sized> {
    pub user_repository: Arc<T>,
}

impl<T: UserRepository + ?Sized> DefaultUserService<T> {
    pub fn new(user_repository: Arc<T>) -> Self {
        Self { user_repository }
    }
}

#[async_trait::async_trait]
impl<T: UserRepository + ?Sized> UserService for DefaultUserService<T> {
    async fn create_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreateUserCommand,
    ) -> shinespark::Result<UserWithIdentities> {
        let user = User::new(
            uuid::Uuid::new_v4(),
            command.name.clone(),
            command.email.clone(),
            command.status,
        );

        let user = self.user_repository.create_user(handle, user).await?;
        let (provider, provider_uid, credential_hash) = match command.credentials {
            InitialCredentials::Local { password } => (
                crate::entity::AuthProvider::Local,
                command.email.clone(),
                Some(password),
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
    use super::*;
    use crate::entity::{AuthProvider, User, UserIdentity, UserStatus, UserWithRoles};
    use crate::repository::{DefaultUserRepository, UserRepository};
    use crate::service::InitialCredentials;
    use std::sync::{Arc, Mutex};

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
            query: super::FindUserQuery,
        ) -> shinespark::Result<Option<UserWithRoles>> {
            let users = self.users.lock().unwrap();
            let user = users.iter().find(|u| {
                let id_match = query.id.map_or(true, |id| u.id == id);
                let uid_match = query.uid.map_or(true, |uid| u.uid == uid);
                let email_match = query.email.as_ref().map_or(true, |email| &u.email == email);
                id_match && uid_match && email_match
            });

            Ok(user.map(|u| UserWithRoles {
                user: u.clone(),
                role_ids: vec![],
            }))
        }

        async fn update_user(
            &self,
            _handle: &mut shinespark::db::Handle<'_>,
            command: super::UpdateUserCommand,
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

    // 헬퍼: DB 사용 여부에 따라 Repository와 Database(존재할 경우) 반환
    async fn setup_test_env() -> (shinespark::db::Database, Arc<dyn UserRepository>) {
        let database = shinespark::db::Database::new_dotenv().await.unwrap();

        let use_mock = true; // 플래그를 통해 Mock 또는 실제 DB 사용 선택

        let user_repository: Arc<dyn UserRepository> = if use_mock {
            Arc::new(MockUserRepository::new())
        } else {
            Arc::new(DefaultUserRepository {})
        };

        (database, user_repository)
    }

    #[tokio::test]
    async fn test_create_user_local() {
        let (database, user_repository) = setup_test_env().await;
        let service = DefaultUserService::new(user_repository.clone());

        let mut tx = database.tx().await.unwrap();

        let command = CreateUserCommand {
            name: "test_local".to_string(),
            email: "test_local@example.com".to_string(),
            credentials: InitialCredentials::Local {
                password: "test_password_hash".to_string(),
            },
            status: UserStatus::Pending,
        };

        let result = service.create_user(&mut tx, command).await;
        assert!(result.is_ok());

        let u = result.unwrap();
        assert_eq!(u.user.name, "test_local");
        assert_eq!(u.user.email, "test_local@example.com");

        // Identity 검증
        assert_eq!(u.identities.len(), 1);
        let identity = &u.identities[0];
        assert_eq!(identity.provider, AuthProvider::Local);
        assert_eq!(
            identity.credential_hash.as_deref(),
            Some("test_password_hash")
        );

        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_create_user_social() {
        let (database, user_repository) = setup_test_env().await;
        let service = DefaultUserService::new(user_repository.clone());

        let mut tx = database.tx().await.unwrap();

        let command = CreateUserCommand {
            name: "test_google".to_string(),
            email: "test_google@example.com".to_string(),
            credentials: InitialCredentials::Social {
                provider: AuthProvider::Google,
                provider_uid: "google_12345".to_string(),
            },
            status: UserStatus::Pending,
        };

        let result = service.create_user(&mut tx, command).await;
        assert!(result.is_ok());

        let u = result.unwrap();
        assert_eq!(u.user.name, "test_google");

        // Identity 검증
        assert_eq!(u.identities.len(), 1);
        let identity = &u.identities[0];
        assert_eq!(identity.provider, AuthProvider::Google);
        assert_eq!(identity.provider_uid, "google_12345");
        assert!(identity.credential_hash.is_none());

        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_find_user() {
        let (database, user_repository) = setup_test_env().await;
        let service = DefaultUserService::new(user_repository.clone());

        let mut tx = database.tx().await.unwrap();

        // 1. 없는 유저 조회 시도
        let not_found = service
            .find_user(
                &mut tx,
                super::FindUserQuery::new().email("non_existent@example.com".to_string()),
            )
            .await;
        assert!(not_found.is_ok());
        assert!(not_found.unwrap().is_none());

        // 2. 유저 생성 후 조회
        let command = CreateUserCommand {
            name: "find_test".to_string(),
            email: "find_test@example.com".to_string(),
            credentials: InitialCredentials::Local {
                password: "hash".to_string(),
            },
            status: UserStatus::Pending,
        };
        let created_user = service.create_user(&mut tx, command).await.unwrap();

        let found = service
            .find_user(
                &mut tx,
                super::FindUserQuery::new().email("find_test@example.com".to_string()),
            )
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().user.id, created_user.user.id);

        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_update_user_status() {
        let (database, user_repository) = setup_test_env().await;
        let service = DefaultUserService::new(user_repository.clone());
        let mut tx = database.tx().await.unwrap();

        // 유저 생성
        let command = CreateUserCommand {
            name: "update_test".to_string(),
            email: "update_test@example.com".to_string(),
            credentials: InitialCredentials::Local {
                password: "hash".to_string(),
            },
            status: UserStatus::Pending,
        };
        let created_user = service.create_user(&mut tx, command).await.unwrap();
        assert_eq!(created_user.user.status, UserStatus::Pending);

        // 업데이트
        let update_command = super::UpdateUserCommand {
            id: created_user.user.id,
            status: Some(UserStatus::Deleted),
        };
        let updated_user = service.update_user(&mut tx, update_command).await.unwrap();

        assert_eq!(updated_user.status, UserStatus::Deleted);

        tx.commit().await.unwrap();
    }
}
