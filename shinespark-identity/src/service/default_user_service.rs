use std::sync::Arc;

use shinespark::crypto::password::PasswordService;

use crate::entity::User;
use crate::repository::UserRepository;
use crate::service::user_service::{CreateUserCommand, UserService};

pub struct DefaultUserService<T: UserRepository, P: PasswordService> {
    pub user_repository: Arc<T>,
    pub password_service: Arc<P>,
}

impl<T: UserRepository, P: PasswordService> DefaultUserService<T, P> {
    pub fn new(user_repository: Arc<T>, password_service: Arc<P>) -> Self {
        Self {
            user_repository,
            password_service,
        }
    }
}

#[async_trait::async_trait]
impl<T: UserRepository, P: PasswordService> UserService for DefaultUserService<T, P> {
    async fn create_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreateUserCommand,
    ) -> shinespark::Result<User> {
        todo!("{:#?} {:#?}", handle, command);
    }
}

#[cfg(test)]
mod tests {
    use shinespark::crypto::password::NoopPasswordService;

    pub struct MockUserRepository {}

    impl UserRepository for MockUserRepository {}

    use super::*;

    #[test]
    fn test_service() {
        let password_service = Arc::new(NoopPasswordService {});
        let user_repository = Arc::new(MockUserRepository {});
        let _service = DefaultUserService::new(user_repository, password_service);
    }
}
