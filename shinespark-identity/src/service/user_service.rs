use crate::entity::{self, User};

// ==========================================
// 1. UserService Cqrs
// ==========================================
#[derive(Debug)]
pub enum InitialCredentials {
    Local {
        password: String,
    },
    Social {
        provider: entity::AuthProvider,
        provider_uid: String,
    },
}

#[derive(Debug)]
pub struct CreateUserCommand {
    pub name: String,
    pub email: String,
    pub credentials: InitialCredentials,
}

// ==========================================
// 1. UserService Trait
// ==========================================
// 사용자의 계정(Identity & Profile) 라이프사이클 관리에 집중합니다.
#[async_trait::async_trait]
pub trait UserService {
    async fn create_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreateUserCommand,
    ) -> shinespark::Result<User>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() {
        let user = CreateUserCommand {
            name: "test".to_string(),
            email: "test".to_string(),
            credentials: InitialCredentials::Local {
                password: "test".to_string(),
            },
        };

        println!("{:#?}", user);
    }
}
