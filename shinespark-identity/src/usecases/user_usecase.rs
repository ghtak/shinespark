use crate::entities::{self, User, UserAggregate, UserWithIdentities};

// ==========================================
// 1. UserUsecase Cqrs
// ==========================================
#[derive(Debug)]
pub enum InitialCredentials {
    Local {
        password: String,
    },
    Social {
        provider: crate::entities::AuthProvider,
        provider_uid: String,
    },
}

#[derive(Debug)]
pub struct CreateUserCommand {
    pub name: String,
    pub email: String,
    pub credentials: InitialCredentials,
    pub status: entities::UserStatus,
}

#[derive(Debug)]
pub struct FindUserQuery {
    pub id: Option<i64>,
    pub uid: Option<uuid::Uuid>,
    pub email: Option<String>,
    pub with_deleted: bool,
}

#[derive(Debug)]
pub struct UpdateUserCommand {
    pub id: i64,
    pub status: Option<entities::UserStatus>,
}

#[derive(Debug)]
pub enum LoginCommand {
    Local {
        email: String,
        password: String,
    },
    Social {
        provider: crate::entities::AuthProvider,
        provider_uid: String,
    },
}

// ==========================================
// 1. UserUsecase Trait
// ==========================================
// 사용자의 계정(Identity & Profile) 라이프사이클 관리에 집중합니다.
#[async_trait::async_trait]
pub trait UserUsecase: Send + Sync + 'static {
    async fn create_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreateUserCommand,
    ) -> shinespark::Result<UserWithIdentities>;

    async fn find_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        query: FindUserQuery,
    ) -> shinespark::Result<Option<UserAggregate>>;

    async fn update_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: UpdateUserCommand,
    ) -> shinespark::Result<User>;

    async fn login(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: LoginCommand,
    ) -> shinespark::Result<UserAggregate>;
}

impl FindUserQuery {
    pub fn new() -> Self {
        Self {
            id: None,
            uid: None,
            email: None,
            with_deleted: false,
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

    pub fn with_deleted(mut self, with_deleted: bool) -> Self {
        self.with_deleted = with_deleted;
        self
    }
}
