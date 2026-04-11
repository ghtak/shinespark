use std::sync::Arc;

use shinespark::crypto::password::PasswordService;

use crate::entities::{AuthProvider, UserAggregate};
use crate::repositories::UserRepository;
use crate::usecases::{LoginCommand, LoginUsecase};

pub struct DefaultLoginUsecase<T: UserRepository + ?Sized, P: PasswordService> {
    pub user_repository: Arc<T>,
    pub password_service: Arc<P>,
}

impl<T: UserRepository + ?Sized, P: PasswordService> DefaultLoginUsecase<T, P> {
    pub fn new(user_repository: Arc<T>, password_service: Arc<P>) -> Self {
        Self {
            user_repository,
            password_service,
        }
    }
}

#[async_trait::async_trait]
impl<T: UserRepository + ?Sized, P: PasswordService> LoginUsecase for DefaultLoginUsecase<T, P> {
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
    use std::sync::Arc;

    use super::*;
    use crate::entities::UserStatus;
    use crate::infra::{DefaultUserUsecase, MockUserRepository, SqlxUserRepository};
    use crate::repositories::UserRepository;
    use crate::usecases::{CreateUserCommand, FindUserQuery, InitialCredentials, UserUsecase};

    async fn setup_test_env() -> (shinespark::db::Database, Arc<dyn UserRepository>) {
        let database = shinespark::db::Database::new_dotenv().await.unwrap();
        let use_mock = false;
        let user_repository: Arc<dyn UserRepository> = if use_mock {
            Arc::new(MockUserRepository::new())
        } else {
            Arc::new(SqlxUserRepository::new())
        };
        (database, user_repository)
    }

    async fn delete_user_if_exists(
        database: &Database,
        user_service: Arc<dyn UserUsecase>,
        email: String,
    ) {
        let mut tx = database.tx().await.unwrap();
        if let Ok(Some(u)) =
            user_service.find_user(&mut tx, FindUserQuery::new().email(email)).await
        {
            let _ = user_service
                .update_user(
                    &mut tx,
                    crate::usecases::UpdateUserCommand {
                        id: u.user.id,
                        status: Some(UserStatus::Deleted),
                    },
                )
                .await;
        }
        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_login_local() {
        let (database, user_repository) = setup_test_env().await;
        let password_service = Arc::new(B64PasswordService::new());
        
        let user_service = Arc::new(DefaultUserUsecase::new(
            user_repository.clone(),
            password_service.clone(),
        ));
        
        let login_service = Arc::new(DefaultLoginUsecase::new(
            user_repository.clone(),
            password_service.clone(),
        ));

        delete_user_if_exists(
            &database,
            user_service.clone(),
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
        let created_user = user_service.create_user(&mut tx, command).await.unwrap();
        assert_eq!(created_user.user.status, UserStatus::Active);

        // 로그인
        let login_command = LoginCommand::Local {
            email: "login_test@example.com".to_string(),
            password: "hash".to_string(),
        };
        let logged_in_user = login_service.login(&mut tx, login_command).await.unwrap();

        assert_eq!(logged_in_user.user.id, created_user.user.id);

        tx.commit().await.unwrap();
    }
}
