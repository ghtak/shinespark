use std::sync::Arc;

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::infra::jwt_service::{JwtService, JwtTokenPair};
use crate::repositories::JwtIdentRepository;
use crate::usecases::{FindUserQuery, JwtIdentUsecase, LoginCommand, LoginUsecase, UserUsecase};

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub struct DefaultJwtIdentUsecase<L, U, J, R>
where
    L: LoginUsecase,
    U: UserUsecase,
    J: JwtService,
    R: JwtIdentRepository,
{
    login_usecase: Arc<L>,
    user_usecase: Arc<U>,
    jwt_service: Arc<J>,
    jwt_repository: Arc<R>,
}

impl<L, U, J, R> DefaultJwtIdentUsecase<L, U, J, R>
where
    L: LoginUsecase,
    U: UserUsecase,
    J: JwtService,
    R: JwtIdentRepository,
{
    pub fn new(
        login_usecase: Arc<L>,
        user_usecase: Arc<U>,
        jwt_service: Arc<J>,
        jwt_repository: Arc<R>,
    ) -> Self {
        Self {
            login_usecase,
            user_usecase,
            jwt_service,
            jwt_repository,
        }
    }
}

#[async_trait::async_trait]
impl<L, U, J, R> JwtIdentUsecase for DefaultJwtIdentUsecase<L, U, J, R>
where
    L: LoginUsecase,
    U: UserUsecase,
    J: JwtService,
    R: JwtIdentRepository,
{
    async fn login(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: LoginCommand,
    ) -> shinespark::Result<JwtTokenPair> {
        let aggregate = self.login_usecase.login(handle, command).await?;
        let pair = self.jwt_service.create(&aggregate)?;
        let token_hash = sha256_hex(&pair.refresh_token);
        self.jwt_repository
            .save_refresh_token(
                handle,
                &aggregate.user.uid.to_string(),
                &token_hash,
                pair.refresh_token_expires_at,
            )
            .await?;
        Ok(pair)
    }

    async fn logout(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_uid: &str,
    ) -> shinespark::Result<()> {
        self.jwt_repository.delete_by_user_uid(handle, user_uid).await
    }

    async fn refresh(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        refresh_token: &str,
    ) -> shinespark::Result<JwtTokenPair> {
        let claims = self.jwt_service.verify(refresh_token)?;
        if claims.token_type != "refresh" {
            return Err(shinespark::Error::UnAuthorized);
        }

        let token_hash = sha256_hex(refresh_token);
        let refresh_token_row = self
            .jwt_repository
            .find_refresh_token(handle, &token_hash)
            .await?
            .ok_or(shinespark::Error::UnAuthorized)?;

        if refresh_token_row.expires_at < Utc::now() {
            return Err(shinespark::Error::UnAuthorized);
        }

        let uid =
            uuid::Uuid::parse_str(&claims.sub).map_err(|_| shinespark::Error::UnAuthorized)?;

        if refresh_token_row.user_uid != uid {
            return Err(shinespark::Error::UnAuthorized);
        }

        let aggregate = self
            .user_usecase
            .find_user(handle, FindUserQuery::new().uid(uid))
            .await?
            .ok_or(shinespark::Error::NotFound)?;

        self.jwt_repository.delete_by_user_uid(handle, &claims.sub).await?;

        let pair = self.jwt_service.create(&aggregate)?;
        let new_hash = sha256_hex(&pair.refresh_token);
        self.jwt_repository
            .save_refresh_token(
                handle,
                &claims.sub,
                &new_hash,
                pair.refresh_token_expires_at,
            )
            .await?;

        Ok(pair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use chrono::{DateTime, Utc};
    use shinespark::crypto::password::B64PasswordService;

    use crate::entities::{UserAggregate, UserStatus};
    use crate::infra::jwt_service::{JwtClaims, JwtTokenPair};
    use crate::infra::{DefaultLoginUsecase, DefaultUserUsecase, MockUserRepository};
    use crate::repositories::{JwtIdentRepository, RefreshTokenRow};
    use crate::usecases::{CreateUserCommand, InitialCredentials, UserUsecase};

    // --- Mock JwtService ---

    struct MockJwtService {
        fail_verify: bool,
        wrong_type: bool,
    }

    impl JwtService for MockJwtService {
        fn create(&self, aggregate: &UserAggregate) -> shinespark::Result<JwtTokenPair> {
            Ok(JwtTokenPair {
                access_token: format!("access.{}", aggregate.user.uid),
                refresh_token: format!("refresh.{}", aggregate.user.uid),
                refresh_token_expires_at: Utc::now() + chrono::Duration::hours(24),
            })
        }

        fn verify(&self, token: &str) -> shinespark::Result<JwtClaims> {
            if self.fail_verify {
                return Err(shinespark::Error::UnAuthorized);
            }
            let token_type = if self.wrong_type {
                "access".to_string()
            } else {
                "refresh".to_string()
            };
            let uid =
                token.strip_prefix("refresh.").unwrap_or("00000000-0000-0000-0000-000000000001");
            Ok(JwtClaims {
                sub: uid.to_string(),
                exp: usize::MAX,
                roles: None,
                token_type,
            })
        }
    }

    // --- Mock JwtIdentRepository ---

    struct MockJwtIdentRepository {
        tokens: Mutex<Vec<(String, String)>>, // (user_uid, token_hash)
    }

    impl MockJwtIdentRepository {
        fn new() -> Self {
            Self {
                tokens: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait::async_trait]
    impl JwtIdentRepository for MockJwtIdentRepository {
        async fn save_refresh_token(
            &self,
            _handle: &mut shinespark::db::Handle<'_>,
            user_uid: &str,
            token_hash: &str,
            _expires_at: DateTime<Utc>,
        ) -> shinespark::Result<()> {
            self.tokens.lock().unwrap().push((user_uid.to_string(), token_hash.to_string()));
            Ok(())
        }

        async fn find_refresh_token(
            &self,
            _handle: &mut shinespark::db::Handle<'_>,
            token_hash: &str,
        ) -> shinespark::Result<Option<RefreshTokenRow>> {
            let tokens = self.tokens.lock().unwrap();
            let found = tokens.iter().any(|(_, h)| h == token_hash);
            if found {
                Ok(Some(RefreshTokenRow {
                    id: 1,
                    user_uid: uuid::Uuid::new_v4(),
                    token_hash: token_hash.to_string(),
                    expires_at: Utc::now() + chrono::Duration::hours(1),
                    created_at: Utc::now(),
                }))
            } else {
                Ok(None)
            }
        }

        async fn delete_by_user_uid(
            &self,
            _handle: &mut shinespark::db::Handle<'_>,
            user_uid: &str,
        ) -> shinespark::Result<()> {
            self.tokens.lock().unwrap().retain(|(uid, _)| uid != user_uid);
            Ok(())
        }
    }

    fn make_mock_usecase() -> (
        Arc<DefaultLoginUsecase<MockUserRepository, B64PasswordService>>,
        Arc<DefaultUserUsecase<MockUserRepository, B64PasswordService>>,
    ) {
        let password_service = Arc::new(B64PasswordService::new());
        let user_repository = Arc::new(MockUserRepository::new());
        let login_usecase = Arc::new(DefaultLoginUsecase::new(
            user_repository.clone(),
            password_service.clone(),
        ));
        let user_usecase = Arc::new(DefaultUserUsecase::new(
            user_repository.clone(),
            password_service.clone(),
        ));
        (login_usecase, user_usecase)
    }

    #[tokio::test]
    async fn test_login_success() {
        let password_service = Arc::new(B64PasswordService::new());
        let user_repository = Arc::new(MockUserRepository::new());
        let login_usecase = Arc::new(DefaultLoginUsecase::new(
            user_repository.clone(),
            password_service.clone(),
        ));
        let user_usecase = Arc::new(DefaultUserUsecase::new(
            user_repository.clone(),
            password_service.clone(),
        ));
        let jwt_service = Arc::new(MockJwtService {
            fail_verify: false,
            wrong_type: false,
        });
        let jwt_repository = Arc::new(MockJwtIdentRepository::new());

        let usecase = DefaultJwtIdentUsecase::new(
            login_usecase,
            user_usecase.clone(),
            jwt_service,
            jwt_repository,
        );

        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let mut handle = db.handle();

        user_usecase
            .create_user(
                &mut handle,
                CreateUserCommand {
                    name: "test".to_string(),
                    email: "jwt_test@example.com".to_string(),
                    credentials: InitialCredentials::Local {
                        password: "pw".to_string(),
                    },
                    status: UserStatus::Active,
                },
            )
            .await
            .unwrap();

        let result = usecase
            .login(
                &mut handle,
                LoginCommand::Local {
                    email: "jwt_test@example.com".to_string(),
                    password: "pw".to_string(),
                },
            )
            .await;
        assert!(result.is_ok());
        let pair = result.unwrap();
        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let (login_usecase, user_usecase) = make_mock_usecase();
        let jwt_service = Arc::new(MockJwtService {
            fail_verify: false,
            wrong_type: false,
        });
        let jwt_repository = Arc::new(MockJwtIdentRepository::new());
        let usecase =
            DefaultJwtIdentUsecase::new(login_usecase, user_usecase, jwt_service, jwt_repository);

        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let mut handle = db.handle();

        let result = usecase
            .login(
                &mut handle,
                LoginCommand::Local {
                    email: "notexist@example.com".to_string(),
                    password: "wrong".to_string(),
                },
            )
            .await;
        assert!(matches!(result, Err(shinespark::Error::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_logout_removes_token() {
        let (login_usecase, user_usecase) = make_mock_usecase();
        let jwt_service = Arc::new(MockJwtService {
            fail_verify: false,
            wrong_type: false,
        });
        let jwt_repository = Arc::new(MockJwtIdentRepository::new());
        let usecase = DefaultJwtIdentUsecase::new(
            login_usecase,
            user_usecase,
            jwt_service,
            jwt_repository.clone(),
        );

        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let mut handle = db.handle();

        let uid = "00000000-0000-0000-0000-000000000099";
        jwt_repository.tokens.lock().unwrap().push((uid.to_string(), "somehash".to_string()));
        usecase.logout(&mut handle, uid).await.unwrap();
        assert!(jwt_repository.tokens.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_refresh_with_invalid_token() {
        let (login_usecase, user_usecase) = make_mock_usecase();
        let jwt_service = Arc::new(MockJwtService {
            fail_verify: true,
            wrong_type: false,
        });
        let jwt_repository = Arc::new(MockJwtIdentRepository::new());
        let usecase =
            DefaultJwtIdentUsecase::new(login_usecase, user_usecase, jwt_service, jwt_repository);

        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let mut handle = db.handle();

        let result = usecase.refresh(&mut handle, "invalid.token").await;
        assert!(matches!(result, Err(shinespark::Error::UnAuthorized)));
    }

    #[tokio::test]
    async fn test_refresh_with_access_token_type() {
        let (login_usecase, user_usecase) = make_mock_usecase();
        let jwt_service = Arc::new(MockJwtService {
            fail_verify: false,
            wrong_type: true,
        });
        let jwt_repository = Arc::new(MockJwtIdentRepository::new());
        let usecase =
            DefaultJwtIdentUsecase::new(login_usecase, user_usecase, jwt_service, jwt_repository);

        let db = shinespark::db::Database::new_dotenv().await.unwrap();
        let mut handle = db.handle();

        let result = usecase.refresh(&mut handle, "some.token").await;
        assert!(matches!(result, Err(shinespark::Error::UnAuthorized)));
    }
}
