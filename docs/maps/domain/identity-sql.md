---
scope: identity crate 의 sql/ 디렉터리 레이아웃, 네이밍, shs_iam_* 테이블 규약
when-to-read: SQL 파일 추가·수정, 스키마·테이블 변경, sqlx_statement enum 확장
budget: 200
related:
  - ../_index.md
  - ../crates/shinespark-identity.md
  - ./identity-repositories.md
updated: 2026-04-18
---

## TL;DR

- 모든 테이블 prefix: **`shs_iam_*`**
- SQL 파일은 **repository 단위 하위 디렉터리** 로 분할: `sql/<feature>_repository/`
- 파일 네이밍은 동사-중심 (`create_user.sql`, `find_user_by_identity.sql`)
- Rust 에서는 `sqlx_statement.rs` 의 enum variant + `include_str!` 로 라우팅
- SQL 방언: **PostgreSQL 기준**. 다른 driver 사용 시 별도 검증 필요

## 디렉터리 레이아웃

```
shinespark-identity/sql/
  user_repository/
    create_user.sql
    create_identity.sql
    find_user.sql
    find_user_by_identity.sql
    update_user.sql
  rbac_repository/
    # permission CRUD
    create_permission.sql
    list_permissions.sql
    find_permission_by_code.sql
    delete_permission.sql
    # role CRUD
    create_role.sql
    list_roles.sql
    find_role_by_name.sql
    delete_role.sql
    # link
    add_permission_to_role.sql
    remove_permission_from_role.sql
    assign_role_to_user.sql
    remove_role_from_user.sql
    # cascade helpers
    delete_role_permissions_by_role_id.sql
    delete_role_permissions_by_permission_id.sql
    delete_user_roles_by_role_id.sql
    # bulk for cache
    load_role_permissions.sql
  jwt_repository/
    save_refresh_token.sql
    find_refresh_token.sql
    delete_by_user_uid.sql
```

규칙:

- **하위 디렉터리 = 하나의 repository trait** (`sql/<feature>_repository/`)
- **파일 하나 = SQL statement 하나** — 여러 쿼리 합치지 않음
- 파일명은 동사 먼저 (`create_`, `find_`, `delete_`, `update_`, `list_`, `add_`, `remove_`, `assign_`, `revoke_`, `load_`)
- 검색 조건을 명시: `find_user.sql` (일반) vs `find_user_by_identity.sql` (특수)
- cascade/helper 는 `..._by_<target>_id` 로 대상 명시

## Rust 쪽 라우팅

`shinespark-identity/src/infra/sqlx_statement.rs` 가 enum 으로 모든 SQL 을 관리:

```rust
pub enum Query {
    CreateUser,
    FindUser,
    // ...
}

impl Query {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CreateUser => include_str!("../../sql/user_repository/create_user.sql"),
            Self::FindUser   => include_str!("../../sql/user_repository/find_user.sql"),
            // ...
        }
    }
}
```

추가 규칙:

- `SqlxXxxRepository::method` 는 `Query::Xxx.as_str()` 로 SQL 획득 후 `sqlx::query*` 에 투입
- variant 이름은 파일명을 CamelCase 로 (`find_user_by_identity.sql` → `FindUserByIdentity`)
- SQL 파일 추가 시 enum variant + match arm 도 같이 추가 — **누락 시 컴파일 타임 에러 발생하지 않음** (런타임 전엔 발견 안 됨) 주의

## 테이블 목록

현재 존재하는 `shs_iam_*` 테이블:

| 테이블 | 용도 |
|---|---|
| `shs_iam_user` | 사용자 기본 정보 (id, uid, name, email, status, timestamps) |
| `shs_iam_user_identity` | provider 별 자격증명 (Local/Google/Apple + credential_hash) |
| `shs_iam_user_audit_log` | 로그인/로그아웃/상태변경 등 감사 로그 |
| `shs_iam_role` | 역할 정의 |
| `shs_iam_permission` | 권한 정의 (code 는 `Resource.action.scope`) |
| `shs_iam_role_permission` | role ↔ permission 링크 |
| `shs_iam_user_role` | user ↔ role 링크 |
| `shs_iam_refresh_token` | JWT refresh token (SHA256 해시 저장) |

구체 컬럼 / 인덱스 정의는 마이그레이션 SQL 참조 (`docs/03. migrations.md`). 컬럼 변경이 있을 경우 영향 파일:

- `entities.rs` 의 struct 필드
- `sqlx_*_repository.rs` 의 `Row` 구조체와 `From` impl
- 관련 SQL 파일들 (`INSERT`, `SELECT` 컬럼 나열)

## SQL 작성 규약

- **식별자**: snake_case. 예약어 사용 시 `"..."` 큰따옴표 (Postgres 관례) — 현재 회피 중
- **파라미터 바인딩**: Postgres 위치 파라미터 `$1, $2, ...` — driver 전환 시 깨질 위험 있음
- **반환 컬럼**: `SELECT *` 금지. 명시 나열 (`Row` 구조체와 일치시키기 위함)
- **JOIN 순서**: aggregate 조회 (`find_user.sql`) 는 user → identity → role_id 순으로 LEFT JOIN + 집계
- **UPSERT**: `INSERT ... ON CONFLICT (...) DO UPDATE ...` 사용 (`save_refresh_token.sql`)
- **Cascade**: FK `ON DELETE CASCADE` 가 아닌 **명시적 delete 쿼리**로 처리 (순서·멱등성을 usecase 가 제어)

## 파일 추가 체크리스트

1. `sql/<feature>_repository/<verb>_<object>[_by_<filter>].sql` 생성
2. `sqlx_statement.rs` 의 enum 에 variant + match arm 추가
3. `Sqlx*Repository` 의 해당 method 가 `Query::Xxx.as_str()` 를 사용하도록 구현
4. 테이블 prefix `shs_iam_*` 준수
5. 컬럼 나열이 `Row` 구조체와 일치하는지 확인

## 다른 DB driver 사용 시

`shinespark` 의 feature flag 로 `db-driver-sqlite` 또는 `db-driver-mysql` 을 택한 경우:

- Postgres 전용 구문 (`ON CONFLICT`, `RETURNING *`, `$N` 바인딩 등) 이 깨진다
- SQL 을 driver 별로 분리하려면 디렉터리를 `sql/postgres/<feature>/` 식으로 재구성 + `sqlx_statement.rs` 에서 `#[cfg(feature = "...")]` 분기 필요
- 현재는 **Postgres 단일 기준** — 다른 driver 지원은 미구현 정책
