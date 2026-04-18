---
scope: identity repository trait / Sqlx*Repository / Mock*Repository / Handle 규약
when-to-read: repository trait 수정, 새 Sqlx/Mock impl 추가, DB 접근 패턴 확인
budget: 250
related:
  - ../_index.md
  - ../crates/shinespark-identity.md
  - ../crates/shinespark-core.md
  - ./identity-usecases.md
  - ./identity-sql.md
updated: 2026-04-18
---

## TL;DR

- 3개 repository trait: `UserRepository` / `RbacRepository` / `JwtIdentRepository`
- 구현체: `Sqlx*Repository` (실 DB) / `Mock*Repository` (in-memory, 현재 User 만)
- 모든 method 는 `&mut shinespark::db::Handle<'_>` 를 첫 인자로 수신
- `handle.inner()` 로 sqlx executor 획득 후 `.fetch_*` / `.execute`
- SQL 본문은 `sql/` 파일 + `sqlx_statement.rs` enum 으로 관리 (`./identity-sql.md`)

## Repository trait

### `UserRepository`
파일: `shinespark-identity/src/repositories/user_repository.rs:UserRepository`

역할: 사용자 + identity 의 CRUD, aggregate 조회, identity 기반 조회.

주요 method (시그니처는 소스 확인):

- `create_user` / `create_identity` — insert
- `find_user` — id/uid/email 조합 조회, roles + identities 포함 (aggregate)
- `find_user_by_identity` — provider + provider_uid 로 조회
- `update_user` — status 변경

### `RbacRepository`
파일: `shinespark-identity/src/repositories/rbac_repository.rs:RbacRepository`

역할: permission/role CRUD, 링크, cascade 삭제, 캐시 적재용 bulk load.

method 군:

| 군 | 용도 |
|---|---|
| Permission CRUD | create / delete / list / find_by_code |
| Role CRUD | create / delete / list / find_by_name |
| Link | add_permission_to_role / remove_permission_from_role |
| Cascade | delete_role_permissions_by_{permission,role}_id / delete_user_roles_by_role_id |
| Bulk | load_role_permissions — 캐시 워밍용 `(role_id, permission_code)` 페어 |
| User-Role | assign_role_to_user / remove_role_from_user |

### `JwtIdentRepository`
파일: `shinespark-identity/src/repositories/jwt_ident_repository.rs:JwtIdentRepository`

역할: refresh token 저장/조회/무효화. 토큰은 SHA256 해시 상태로 보관.

- `save_refresh_token` — upsert (token_hash 기준)
- `find_refresh_token` — token_hash 로 조회
- `delete_by_user_uid` — 사용자별 전체 무효화

## Sqlx impl 규약

파일: `shinespark-identity/src/infra/sqlx_<feature>_repository.rs`

구조 (대표):

```rust
pub struct SqlxXxxRepository;

#[async_trait::async_trait]
impl XxxRepository for SqlxXxxRepository {
    async fn find_something(
        &self,
        handle: &mut Handle<'_>,
        id: i64,
    ) -> Result<Option<Xxx>> {
        let executor = handle.inner();
        sqlx::query_as::<_, XxxRow>(Query::FindSomething.as_str())
            .bind(id)
            .fetch_optional(executor)
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))
    }
}
```

주요 규약:

- **State 없음** — struct 는 보통 unit. 모든 의존성은 인자 `handle` 로 주입
- **SQL inline 금지** — `sqlx_statement.rs` enum variant 로 라우팅
- **Row → entity 변환** — DB row 타입 (`XxxRow`) 을 별도 struct 로 두고 `From` 구현
- **에러 매핑** — sqlx::Error → `shinespark::Error::DatabaseError`
- **conflict 처리** — insert 시 unique 충돌은 `AlreadyExists` 로 명시 매핑

## Mock impl 규약

파일: `shinespark-identity/src/infra/mock_<feature>_repository.rs`

현재는 `MockUserRepository` 하나만 존재. 패턴:

```rust
pub struct MockUserRepository {
    users: Mutex<HashMap<i64, User>>,
    identities: Mutex<HashMap<i64, Vec<UserIdentity>>>,
    // ...
}
```

- `Mutex<HashMap<...>>` 기반 in-memory
- `Default` impl 으로 손쉬운 생성
- `trait` 시그니처는 `Sqlx*` 와 동일 — 테스트에서 `Arc<dyn XxxRepository>` 자리에 그대로 대체 가능
- 단위 테스트는 `Default*Usecase` + Mock repo 조합으로 작성

## Handle 사용 패턴

`Handle` 은 3-variant (Pool / Tx / Conn). repository 관점에서는:

- 대개 `&mut Handle<'_>` 를 그대로 받아 `.inner()` 로 executor 획득
- 트랜잭션 경계는 **usecase 가 결정** — `Database::tx()` 로 `Handle::Tx` 생성 후 repository 여러 개 호출, 마지막에 `.commit()`
- repository 내부에서 새 트랜잭션을 열지 않는다 (호출자 결정)

자세한 `Handle` 동작은 `../crates/shinespark-core.md:Handle<'c>` 섹션.

## 새 repository 추가 순서

1. `repositories/<feature>_repository.rs` 에 trait 선언
2. `repositories/mod.rs` 에 `pub use` 재노출
3. `infra/sqlx_<feature>_repository.rs` 작성 — unit struct + `impl XxxRepository`
4. `infra/sqlx_statement.rs` 의 enum 에 variant 추가 (또는 feature 전용 enum 신설)
5. `sql/<feature>_repository/*.sql` 파일 생성 — 네이밍은 `./identity-sql.md`
6. 필요 시 `infra/mock_<feature>_repository.rs` 작성 (단위 테스트용)

## 주의

- sqlx driver 는 feature flag 로 단일 선택됨 — SQL 은 Postgres 방언 기준 작성 (`./identity-sql.md`)
- `find_*` 계열은 `Option<T>` 반환, 존재하지 않으면 `None`. `NotFound` 에러로 올릴지는 usecase 결정
- bulk 삭제 (cascade) 는 단일 트랜잭션으로 묶어야 안전 — usecase 에서 `Database::tx()` 사용
