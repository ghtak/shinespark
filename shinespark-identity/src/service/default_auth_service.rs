use crate::{
    entity::{UserWithIdentities, UserWithRoles},
    service::{
        AuthService, CreateUserCommand, FindUserQuery, InitialCredentials, RbacService,
        UpdateUserCommand, UserService,
    },
};
use shinespark::crypto::password::PasswordService;
use std::sync::Arc;
// 추후 PasswordService도 UserService가 아닌 AuthService로 주입받게 될 수 있습니다.
pub struct DefaultAuthService<U: UserService, R: RbacService, P: PasswordService> {
    user_service: Arc<U>,
    rbac_service: Arc<R>,
    password_service: Arc<P>,
}

impl<U: UserService, R: RbacService, P: PasswordService> DefaultAuthService<U, R, P> {
    pub fn new(user_service: Arc<U>, rbac_service: Arc<R>, password_service: Arc<P>) -> Self {
        Self {
            user_service,
            rbac_service,
            password_service,
        }
    }
}

#[async_trait::async_trait]
impl<U: UserService, R: RbacService, P: PasswordService> AuthService
    for DefaultAuthService<U, R, P>
{
    async fn sign_up(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: crate::service::CreateUserCommand,
    ) -> shinespark::Result<UserWithIdentities> {
        if self
            .user_service
            .find_user(handle, FindUserQuery::new().email(command.email.clone()))
            .await?
            .is_some()
        {
            return Err(shinespark::Error::AlreadyExists);
        }

        let rebind_command = match command.credentials {
            InitialCredentials::Local { password } => CreateUserCommand {
                name: command.name,
                email: command.email,
                credentials: InitialCredentials::Local {
                    password: self.password_service.hash_password(password.as_bytes())?,
                },
            },
            _ => command,
        };

        let mut user = self
            .user_service
            .create_user(handle, rebind_command)
            .await?;
        let _ = self
            .user_service
            .update_user(
                handle,
                UpdateUserCommand {
                    id: user.user.id,
                    status: Some(crate::entity::UserStatus::Active),
                },
            )
            .await?;
        user.user.status = crate::entity::UserStatus::Active;
        Ok(user)
    }

    // async fn login(
    //     &self,
    //     _handle: &mut shinespark::db::Handle<'_>,
    //     _command: crate::service::LoginCommand,
    // ) -> shinespark::Result<UserWithRoles> {
    //     todo!()
    // }

    // async fn logout(
    //     &self,
    //     _handle: &mut shinespark::db::Handle<'_>,
    //     _user_id: i64,
    // ) -> shinespark::Result<()> {
    //     todo!()
    // }
}

#[cfg(test)]
mod tests {
    use crate::repository::DefaultUserRepository;
    use crate::service::{DefaultRbacService, DefaultUserService};
    use shinespark::crypto::password::B64PasswordService;

    use super::*;

    #[tokio::test]
    async fn test_create_user() {
        let database = shinespark::db::Database::new_dotenv().await.unwrap();
        let password_service = Arc::new(B64PasswordService {});
        let user_repository = Arc::new(DefaultUserRepository::new());
        let user_service = Arc::new(DefaultUserService::new(user_repository));
        let rbac_service = Arc::new(DefaultRbacService::new());
        let auth_service = Arc::new(DefaultAuthService::new(
            user_service.clone(),
            rbac_service.clone(),
            password_service.clone(),
        ));

        let cmd = CreateUserCommand {
            name: "test".to_string(),
            email: "test".to_string(),
            credentials: InitialCredentials::Local {
                password: "test".to_string(),
            },
        };

        if let Some(user) = user_service
            .find_user(
                &mut database.handle(),
                FindUserQuery::new().email(cmd.email.clone()),
            )
            .await
            .unwrap()
        {
            user_service
                .update_user(
                    &mut database.handle(),
                    UpdateUserCommand {
                        id: user.user.id,
                        status: Some(crate::entity::UserStatus::Deleted),
                    },
                )
                .await
                .unwrap();
        }

        let mut tx = database.tx().await.unwrap();
        let result = auth_service
            .sign_up(
                &mut tx,
                CreateUserCommand {
                    name: "test".to_string(),
                    email: "test".to_string(),
                    credentials: InitialCredentials::Local {
                        password: "test".to_string(),
                    },
                },
            )
            .await
            .unwrap();
        println!("{:?}", result);
    }
}
