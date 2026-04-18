---
scope: shinespark-identity (도메인) crate 의 기능 인벤토리와 하위 map 진입점
when-to-read: identity 도메인 작업 진입 — 구체 작업은 하위 domain/*.md 로 이동
budget: 250
related:
  - ../_index.md
  - ../architecture.md
  - ../domain/identity-usecases.md
  - ../domain/identity-repositories.md
  - ../domain/identity-sql.md
updated: 2026-04-18
---

## TL;DR

- 기능 군 4개: **User**, **Login**, **RBAC**, **JwtIdent**
- 레이어: `entities` → `usecases/` (trait) → `repositories/` (trait) → `infra/` (impl)
- 테이블 prefix **`shs_iam_*`** — 다른 도메인과 네임스페이스 분리
- JWT 는 HS256, access + refresh 토큰 쌍
- 부팅 시 `seed_admin` 이 개발용 admin 계정 보장

## 기능 인벤토리

| 기능 | Usecase trait | 대표 method |
|---|---|---|
| User 생명주기 | `UserUsecase` | `create_user` / `find_user` / `update_user` |
| 로그인 (세션용) | `LoginUsecase` | `login` — `UserAggregate` 반환 |
| RBAC | `RbacUsecase` | `check_perm` (sync) + CRUD + 역할·권한 링크 |
| JWT 기반 인증 | `JwtIdentUsecase` | `login` / `refresh` / `logout` — token pair 반환 |

세부 시그니처와 DTO 는 `../domain/identity-usecases.md`.

## 모듈 트리

```
shinespark-identity/src/
  entities.rs             # User, UserIdentity, UserAggregate, Role, Permission 등
  usecases/               # trait 정의 (port)
    user_usecase.rs
    login_usecase.rs
    rbac_usecase.rs
    jwt_ident_usecase.rs
  repositories/           # repository trait 정의
    user_repository.rs
    rbac_repository.rs
    jwt_ident_repository.rs
  infra/                  # trait impl (adapter)
    default_user_usecase.rs
    default_login_usecase.rs
    default_rbac_usecase.rs
    default_jwt_ident_usecase.rs
    sqlx_user_repository.rs
    sqlx_rbac_repository.rs
    sqlx_jwt_ident_repository.rs
    mock_user_repository.rs
    jwt_service.rs         # HS256JwtService + JwtService trait
    seed_user.rs           # seed_admin
    sqlx_statement.rs      # SQL enum (include_str! 라우팅)
sql/                      # SQL 파일 (자세한 규약은 domain/identity-sql.md)
  user_repository/
  rbac_repository/
  jwt_repository/
```

## Entities 핵심

| 타입 | 파일 | 역할 |
|---|---|---|
| `User` | `entities.rs` | 최소 사용자 (id, uid, email, name, status, 타임스탬프) |
| `UserIdentity` | `entities.rs` | 다중 provider 자격증명 (Local/Google/Apple) |
| `UserAggregate` | `entities.rs` | `User` + `Vec<UserIdentity>` + `role_ids: Vec<i64>` |
| `UserStatus` enum | `entities.rs` | Active / Inactive / Pending / Suspended / Deleted |
| `AuthProvider` enum | `entities.rs` | Local / Google / Apple |
| `Role`, `Permission` | `entities.rs` | RBAC 기본 엔티티 |
| `UserAuditLog` | `entities.rs` | 로그인 시도 등 감사 로그 |

필드 상세는 소스 참조.

## Infra 개괄

### Default Usecase impl

`infra/default_<feature>_usecase.rs` — 비즈니스 로직 담당. 필드로 `Arc<dyn XxxRepository>` + 필요한 서비스 (`Arc<dyn PasswordService>`, `Arc<dyn JwtService>` 등) 를 조립.

자세한 조립 규칙은 `../domain/identity-usecases.md`.

### Sqlx Repository impl

`infra/sqlx_<feature>_repository.rs` — SQL 실행만 담당. `sqlx_statement.rs` 의 enum 을 통해 `include_str!` 된 SQL 파일에 라우팅.

자세한 규약은 `../domain/identity-repositories.md` 와 `../domain/identity-sql.md`.

### Mock Repository

`infra/mock_user_repository.rs` — `Mutex<HashMap<...>>` 기반. 단위 테스트용 in-memory. 현재 User 만 존재.

### JWT

- `infra/jwt_service.rs:JwtService` trait + `HS256JwtService` impl
- access token TTL, refresh token TTL 은 `JwtConfig` (기본 1800s / 86400s)
- refresh token 은 SHA256 해시로 DB 저장 (`shs_iam_refresh_token`)

### Seed

`infra/seed_user.rs:seed_admin` — 부팅 시 `admin@shinespark.dev` / `password` 계정이 없으면 생성. dev 편의. 배포 전 제거/교체 필요.

## RBAC 동작 요약

- Permission code 포맷: `Resource.action.scope` (예: `users.read.own`)
- 와일드카드: `*.*.all` 은 모든 권한을 부여
- `DefaultRbacUsecase::load` 가 부팅 시 `HashMap<role_id, HashSet<permission_code>>` 로 캐시
- `check_perm(role_ids, code)` 는 sync (캐시 조회)
- Role 삭제는 cascade: `user_role` + `role_permission` 도 제거
- Permission 삭제는 `role_permission` cascade
- 전체 method 표는 `../domain/identity-usecases.md`, 캐시 무효화는 거기 참고

## Table prefix 와 스키마

모든 테이블은 `shs_iam_*`. 주요 테이블: `shs_iam_user`, `shs_iam_user_identity`, `shs_iam_role`, `shs_iam_permission`, `shs_iam_role_permission`, `shs_iam_user_role`, `shs_iam_refresh_token`, `shs_iam_user_audit_log`.

스키마와 컬럼 상세는 마이그레이션 SQL 과 `docs/03. migrations.md`, 파일 조직은 `../domain/identity-sql.md`.

## 확장 시

- 새 identity-기능 추가: `../feature-lifecycle.md` 레시피 그대로
- 새 도메인 crate 분리: `../architecture.md` 마지막 섹션
