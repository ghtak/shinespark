use crate::entities::{Permission, Role};

// ==========================================
// 1. RBAC Usecase Cqrs
// ==========================================

#[derive(Debug)]
pub struct CreatePermissionCommand {
    pub code: String,
    pub description: String,
}

#[derive(Debug)]
pub struct CreateRoleCommand {
    pub name: String,
    pub description: String,
}

// ==========================================
// 2. RBAC Usecase Trait
// ==========================================
#[async_trait::async_trait]
pub trait RbacUsecase: Send + Sync + 'static {
    /// 서버 구동 시 DB에서 role_id → permission_codes 캐시를 적재한다.
    async fn load(&self, handle: &mut shinespark::db::Handle<'_>) -> shinespark::Result<()>;

    /// 동기 권한 검사 — 메모리 캐시만 사용, DB 접근 없음.
    /// role_ids 중 하나라도 해당 permission 또는 `*.*.all` 을 보유하면 true.
    fn check_perm(&self, role_ids: &[i64], permission: &str) -> bool;

    // ── Permission CRUD ──────────────────────────────────────────────────────

    /// 새 Permission을 생성한다. 동일 code가 이미 존재하면 `AlreadyExists`.
    async fn create_permission(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreatePermissionCommand,
    ) -> shinespark::Result<Permission>;

    /// Permission을 삭제한다. role_permission cascade 후 삭제 → 캐시 갱신.
    async fn delete_permission(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        permission_id: i64,
    ) -> shinespark::Result<()>;

    async fn list_permissions(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<Permission>>;

    async fn find_permission_by_code(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        code: &str,
    ) -> shinespark::Result<Option<Permission>>;

    // ── Role CRUD ────────────────────────────────────────────────────────────

    /// 새 Role을 생성한다. 동일 name이 이미 존재하면 `AlreadyExists`.
    async fn create_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: CreateRoleCommand,
    ) -> shinespark::Result<Role>;

    /// Role을 삭제한다. user_role + role_permission cascade 후 삭제 → 캐시 갱신.
    async fn delete_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
    ) -> shinespark::Result<()>;

    async fn list_roles(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
    ) -> shinespark::Result<Vec<Role>>;

    // ── 역할-권한 링크 (기존 ID 기반) ────────────────────────────────────────

    /// role_id에 permission_id를 추가하고 캐시를 갱신한다.
    async fn add_permission_to_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()>;

    /// role_id에서 permission_id를 제거하고 캐시를 갱신한다.
    async fn remove_permission_from_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()>;

    // ── 역할-권한 링크 (code/name 기반 상위 레벨) ────────────────────────────

    /// role_name + permission_code로 링크를 추가한다. 존재하지 않으면 `NotFound`.
    async fn assign_permission_to_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_name: &str,
        permission_code: &str,
    ) -> shinespark::Result<()>;

    /// role_name + permission_code로 링크를 제거한다. 존재하지 않으면 `NotFound`.
    async fn revoke_permission_from_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_name: &str,
        permission_code: &str,
    ) -> shinespark::Result<()>;

    // ── 유저-역할 관리 ────────────────────────────────────────────────────────

    /// role_name으로 역할을 조회하여 user_id에 부여한다.
    async fn assign_role_to_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
        role_name: &str,
    ) -> shinespark::Result<()>;
}
