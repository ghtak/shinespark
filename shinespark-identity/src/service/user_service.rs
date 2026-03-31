use crate::entity::{UserWithIdentities, UserWithRoles};

// ==========================================
// 1. UserService Cqrs
// ==========================================
#[derive(Debug)]
pub enum InitialCredentials {
    Local {
        password: String,
    },
    Social {
        provider: crate::entity::AuthProvider,
        provider_uid: String,
    },
}

#[derive(Debug)]
pub struct CreateUserCommand {
    pub name: String,
    pub email: String,
    pub credentials: InitialCredentials,
}

#[derive(Debug)]
pub struct FindUserQuery {
    pub id: Option<i64>,
    pub uid: Option<uuid::Uuid>,
    pub email: Option<String>,
}

impl FindUserQuery {
    pub fn new() -> Self {
        Self {
            id: None,
            uid: None,
            email: None,
        }
    }

    pub fn id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn uid(mut self, uid: uuid::Uuid) -> Self {
        self.uid = Some(uid);
        self
    }

    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
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
    ) -> shinespark::Result<UserWithIdentities>;

    async fn find_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        query: FindUserQuery,
    ) -> shinespark::Result<Option<UserWithRoles>>;
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
