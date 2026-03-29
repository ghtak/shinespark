use crate::entity::{self, Permission, Role, User};

// ==========================================
// 1. UserService DTOs
// ==========================================
pub struct CreateUserCommand {
    pub name: String,
    pub email: String,
    pub credentials: InitialCredentials,
}

pub enum InitialCredentials {
    Local {
        password_hash: String,
    },
    Social {
        provider: entity::AuthProvider,
        provider_uid: String,
    },
}

pub struct GetUserQuery {
    pub id: u64,
}

pub struct FindUsersQuery {
    pub name: Option<String>,
    pub email: Option<String>,
    pub status: Option<entity::Status>,
}

pub struct UpdateUserCommand {
    pub user_id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub status: Option<entity::Status>,
}

pub struct DeleteUserCommand {
    pub user_id: u64,
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

    async fn find_users(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        query: FindUsersQuery,
    ) -> shinespark::Result<Vec<User>>;

    async fn get_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        query: GetUserQuery,
    ) -> shinespark::Result<User>;

    async fn update_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: UpdateUserCommand,
    ) -> shinespark::Result<User>;

    async fn delete_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: DeleteUserCommand,
    ) -> shinespark::Result<User>;
}

// ==========================================
// 2. RbacService DTOs
// ==========================================
pub struct AssignRoleToUserCommand {
    pub user_id: u64,
    pub role_id: u64,
}

pub struct RevokeRoleFromUserCommand {
    pub user_id: u64,
    pub role_id: u64,
}

pub struct CreateRoleCommand {
    pub name: String,
    pub description: String,
}

pub struct CreatePermissionCommand {
    pub code: String,
    pub description: String,
}

pub struct AddPermissionToRoleCommand {
    pub role_id: u64,
    pub permission_id: u64,
}

pub struct RemovePermissionFromRoleCommand {
    pub role_id: u64,
    pub permission_id: u64,
}

// ==========================================
// 2. RbacService Trait
// ==========================================
// 권한 및 역할 기초 데이터 정의와, 사용자에 대한 권한 할당을 담당합니다.
#[async_trait::async_trait]
pub trait RbacService {
    // 기초 엔티티 생성
    async fn create_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreateRoleCommand,
    ) -> shinespark::Result<Role>;

    async fn create_permission(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreatePermissionCommand,
    ) -> shinespark::Result<Permission>;

    // 역할에 권한 부여 및 회수
    async fn add_permission_to_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: AddPermissionToRoleCommand,
    ) -> shinespark::Result<Role>;

    async fn remove_permission_from_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: RemovePermissionFromRoleCommand,
    ) -> shinespark::Result<Role>;

    // 사용자에게 역할 부여 및 회수
    async fn assign_role_to_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: AssignRoleToUserCommand,
    ) -> shinespark::Result<()>;

    async fn revoke_role_from_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: RevokeRoleFromUserCommand,
    ) -> shinespark::Result<()>;
}
