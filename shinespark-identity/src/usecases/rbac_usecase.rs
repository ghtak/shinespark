// ==========================================
// 1. RBAC Usecase Cqrs
// ==========================================

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

    /// role_name으로 역할을 조회하여 user_id에 부여한다.
    async fn assign_role_to_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_id: i64,
        role_name: &str,
    ) -> shinespark::Result<()>;

    /// 역할에 권한을 추가하고 캐시를 갱신한다.
    async fn add_permission_to_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()>;

    /// 역할에서 권한을 제거하고 캐시를 갱신한다.
    async fn remove_permission_from_role(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        role_id: i64,
        permission_id: i64,
    ) -> shinespark::Result<()>;
}
