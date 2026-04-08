use std::sync::Arc;

use shinespark::crypto::password::PasswordService;

use crate::entity::{AuthProvider, User, UserAggregate, UserIdentity, UserWithIdentities};
use crate::repository::UserRepository;
use crate::service::{
    CreateUserCommand, FindUserQuery, InitialCredentials, LoginCommand, UpdateUserCommand,
    UserService,
};

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

        let user = self
            .user_repository
            .create_user(
                handle,
                User::new(command.name, command.email, command.status),
            )
            .await?;

        let user_identity = self
            .user_repository
            .create_identity(
                handle,
                UserIdentity::new(user.id, provider, provider_uid, credential_hash),
            )
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
    ) -> shinespark::Result<Option<UserAggregate>> {
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

    async fn login(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: LoginCommand,
    ) -> shinespark::Result<UserAggregate> {
        match command {
            LoginCommand::Local { email, password } => {
                if let Some(user) = self
                    .user_repository
                    .find_user_by_identity(handle, AuthProvider::Local, email)
                    .await?
                {
                    let identity = user
                        .identities
                        .iter()
                        .find(|identity| identity.provider == AuthProvider::Local)
                        .ok_or(shinespark::Error::InvalidCredentials)?;

                    if identity.credential_hash.is_none() {
                        return Err(shinespark::Error::InvalidCredentials);
                    }

                    if self
                        .password_service
                        .verify_password(
                            password.as_bytes(),
                            identity.credential_hash.as_deref().unwrap(),
                        )
                        .is_ok()
                    {
                        return Ok(user);
                    }
                }
                Err(shinespark::Error::InvalidCredentials)
            }
            LoginCommand::Social {
                provider,
                provider_uid,
            } => {
                if let Some(user) = self
                    .user_repository
                    .find_user_by_identity(handle, provider, provider_uid)
                    .await?
                {
                    Ok(user)
                } else {
                    Err(shinespark::Error::NotFound)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use shinespark::crypto::password::B64PasswordService;
    use shinespark::db::Database;

    use super::*;
    use crate::entity::{AuthProvider, UserStatus};
    use crate::infra::{MockUserRepository, SqlxUserRepository};
    use crate::repository::UserRepository;
    use crate::service::InitialCredentials;
    use std::sync::Arc;

    // 헬퍼: DB 사용 여부에 따라 Repository와 Database(존재할 경우) 반환
    async fn setup_test_env() -> (shinespark::db::Database, Arc<dyn UserRepository>) {
        let database = shinespark::db::Database::new_dotenv().await.unwrap();
        let use_mock = false; // 플래그를 통해 Mock 또는 실제 DB 사용 선택
        let user_repository: Arc<dyn UserRepository> = if use_mock {
            Arc::new(MockUserRepository::new())
        } else {
            Arc::new(SqlxUserRepository::new())
        };
        (database, user_repository)
    }

    #[tokio::test]
    async fn test_create_user_local() {
        let (database, user_repository) = setup_test_env().await;
        let password_service = Arc::new(B64PasswordService::new());
        let service = Arc::new(DefaultUserService::new(
            user_repository.clone(),
            password_service.clone(),
        ));

        delete_user_if_exists(
            &database,
            service.clone(),
            "test_local@example.com".to_string(),
        )
        .await;

        let mut tx = database.tx().await.unwrap();

        let command = CreateUserCommand {
            name: "test_local".to_string(),
            email: "test_local@example.com".to_string(),
            credentials: InitialCredentials::Local {
                password: "test_password_hash".to_string(),
            },
            status: UserStatus::Active,
        };

        let result = service.create_user(&mut tx, command).await;
        println!("result: {:?}", result);
        assert!(result.is_ok());
        let u = result.unwrap();
        assert_eq!(u.user.name, "test_local");
        assert_eq!(u.user.email, "test_local@example.com");

        // Identity 검증
        assert_eq!(u.identities.len(), 1);
        let identity = &u.identities[0];
        assert_eq!(identity.provider, AuthProvider::Local);
        let verify = password_service.verify_password(
            "test_password_hash".as_bytes(),
            identity.credential_hash.as_deref().unwrap(),
        );
        assert!(verify.is_ok());
        tx.commit().await.unwrap();
    }

    async fn delete_user_if_exists(
        database: &Database,
        user_service: Arc<dyn UserService>,
        email: String,
    ) {
        let mut tx = database.tx().await.unwrap();
        if let Ok(Some(u)) = user_service
            .find_user(&mut tx, FindUserQuery::new().email(email))
            .await
        {
            let _ = user_service
                .update_user(
                    &mut tx,
                    UpdateUserCommand {
                        id: u.user.id,
                        status: Some(UserStatus::Deleted),
                    },
                )
                .await;
        }
        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_create_user_social() {
        let (database, user_repository) = setup_test_env().await;
        let password_service = Arc::new(B64PasswordService::new());
        let service = Arc::new(DefaultUserService::new(
            user_repository.clone(),
            password_service,
        ));

        let command = CreateUserCommand {
            name: "test_google".to_string(),
            email: "test_google@example.com".to_string(),
            credentials: InitialCredentials::Social {
                provider: AuthProvider::Google,
                provider_uid: "google_12345".to_string(),
            },
            status: UserStatus::Active,
        };

        delete_user_if_exists(&database, service.clone(), command.email.clone()).await;

        let mut tx = database.tx().await.unwrap();

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
        let password_service = Arc::new(B64PasswordService::new());
        let service = Arc::new(DefaultUserService::new(
            user_repository.clone(),
            password_service,
        ));

        delete_user_if_exists(
            &database,
            service.clone(),
            "find_test@example.com".to_string(),
        )
        .await;

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
            status: UserStatus::Active,
        };
        let created_user = service.create_user(&mut tx, command).await.unwrap();

        let found = service
            .find_user(
                &mut tx,
                super::FindUserQuery::new().email("find_test@example.com".to_string()),
            )
            .await
            .unwrap();
        println!("found: {:?}", found);
        assert!(found.is_some());
        assert_eq!(found.unwrap().user.id, created_user.user.id);

        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_update_user_status() {
        let (database, user_repository) = setup_test_env().await;
        let password_service = Arc::new(B64PasswordService::new());
        let service = Arc::new(DefaultUserService::new(
            user_repository.clone(),
            password_service,
        ));

        delete_user_if_exists(
            &database,
            service.clone(),
            "update_test@example.com".to_string(),
        )
        .await;

        let mut tx = database.tx().await.unwrap();

        // 유저 생성
        let command = CreateUserCommand {
            name: "update_test".to_string(),
            email: "update_test@example.com".to_string(),
            credentials: InitialCredentials::Local {
                password: "hash".to_string(),
            },
            status: UserStatus::Active,
        };
        let created_user = service.create_user(&mut tx, command).await.unwrap();
        assert_eq!(created_user.user.status, UserStatus::Active);

        // 업데이트
        let update_command = super::UpdateUserCommand {
            id: created_user.user.id,
            status: Some(UserStatus::Deleted),
        };
        let updated_user = service.update_user(&mut tx, update_command).await.unwrap();

        assert_eq!(updated_user.status, UserStatus::Deleted);

        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_login_local() {
        let (database, user_repository) = setup_test_env().await;
        let password_service = Arc::new(B64PasswordService::new());
        let service = Arc::new(DefaultUserService::new(
            user_repository.clone(),
            password_service.clone(),
        ));

        delete_user_if_exists(
            &database,
            service.clone(),
            "login_test@example.com".to_string(),
        )
        .await;

        let mut tx = database.tx().await.unwrap();

        // 유저 생성
        let command = CreateUserCommand {
            name: "login_test".to_string(),
            email: "login_test@example.com".to_string(),
            credentials: InitialCredentials::Local {
                password: "hash".to_string(),
            },
            status: UserStatus::Active,
        };
        let created_user = service.create_user(&mut tx, command).await.unwrap();
        assert_eq!(created_user.user.status, UserStatus::Active);

        // 로그인
        let login_command = super::LoginCommand::Local {
            email: "login_test@example.com".to_string(),
            password: "hash".to_string(),
        };
        let logged_in_user = service.login(&mut tx, login_command).await.unwrap();

        assert_eq!(logged_in_user.user.id, created_user.user.id);

        tx.commit().await.unwrap();
    }
}
