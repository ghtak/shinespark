---
scope: identity usecase trait / DTO / Default*Usecase 조립 규칙
when-to-read: usecase 추가·수정, DTO 변경, Default*Usecase 내부 조립 이해
budget: 300
related:
  - ../_index.md
  - ../crates/shinespark-identity.md
  - ../feature-lifecycle.md
  - ./identity-repositories.md
updated: 2026-04-18
---

## TL;DR

- 4개 usecase trait: `UserUsecase` / `LoginUsecase` / `RbacUsecase` / `JwtIdentUsecase`
- 모든 async method 의 첫 인자는 `&mut shinespark::db::Handle<'_>`
- DTO 네이밍: `<Action>Command` / `<Query>Query` — trait 파일 안에 동거
- `Default*Usecase` 는 `infra/` 에 있으며 repository + service 를 `Arc<dyn Trait>` 로 조립
- 비즈니스 규칙 (검증·조합·트랜잭션 경계) 은 `Default*Usecase` 에만

## Usecase trait 요약

각 trait 의 전체 시그니처는 소스 직접 참조. 여기서는 의도만.

### `UserUsecase`
파일: `shinespark-identity/src/usecases/user_usecase.rs:UserUsecase`

| Method | 의도 |
|---|---|
| `create_user` | `CreateUserCommand` → `UserWithIdentities`. Local 비밀번호 또는 Social provider_uid 로 계정 + identity 생성 |
| `find_user` | `FindUserQuery` (id/uid/email, with_deleted) → `Option<UserAggregate>` (roles + identities 포함) |
| `update_user` | `UpdateUserCommand` (id + optional status) → `User`. 현재는 status 만 변경 |

DTO:

- `CreateUserCommand` { name, email, credentials, status }
- `InitialCredentials` enum: `Local { password }` / `Social { provider, provider_uid }`
- `FindUserQuery` — builder 패턴 지원 (`.id()`, `.uid()`, `.email()`, `.with_deleted()`)
- `UpdateUserCommand` { id, status: Option<UserStatus> }

### `LoginUsecase`
파일: `shinespark-identity/src/usecases/login_usecase.rs:LoginUsecase`

| Method | 의도 |
|---|---|
| `login` | `LoginCommand` → `UserAggregate`. Local (email + password) 또는 Social (provider + provider_uid) 검증 |

DTO:

- `LoginCommand` enum: `Local { email, password }` / `Social { provider, provider_uid }`

### `RbacUsecase`
파일: `shinespark-identity/src/usecases/rbac_usecase.rs:RbacUsecase`

14개 method. 크게 4군:

| 군 | Method |
|---|---|
| 부팅/검사 | `load` (async, 캐시 워밍), `check_perm` (**sync**, DB 접근 없음) |
| Permission CRUD | `create_permission` / `delete_permission` / `list_permissions` / `find_permission_by_code` |
| Role CRUD | `create_role` / `delete_role` / `list_roles` |
| Link (ID 기반) | `add_permission_to_role` / `remove_permission_from_role` |
| Link (name/code 기반) | `assign_permission_to_role` / `revoke_permission_from_role` |
| User-Role | `assign_role_to_user` |

DTO:

- `CreatePermissionCommand` { code, description }
- `CreateRoleCommand` { name, description }

특이사항:

- `check_perm(role_ids, permission)` 은 **sync** — 메모리 캐시만 조회. DB 접근 시 caller 실수
- Role 삭제 → `user_role` + `role_permission` cascade 후 캐시 갱신
- Permission 삭제 → `role_permission` cascade 후 캐시 갱신
- 와일드카드 `*.*.all` 은 모든 permission 을 통과시킴

### `JwtIdentUsecase`
파일: `shinespark-identity/src/usecases/jwt_ident_usecase.rs:JwtIdentUsecase`

| Method | 의도 |
|---|---|
| `login` | `LoginCommand` (공유 DTO) → `JwtTokenPair`. 내부적으로 `LoginUsecase` 호출 후 토큰 발급 + refresh 저장 |
| `refresh` | `refresh_token: &str` → 새 `JwtTokenPair`. 기존 refresh 검증 후 rotation |
| `logout` | `user_uid: &str` → 해당 사용자의 refresh token 전부 무효화 |

`JwtTokenPair` 는 `shinespark-identity/src/infra/jwt_service.rs:JwtTokenPair` (access_token, refresh_token, expires_at 필드).

## Default*Usecase 조립 규칙

모든 default impl 은 `shinespark-identity/src/infra/default_<feature>_usecase.rs`.

공통 패턴:

```rust
pub struct DefaultXxxUsecase {
    repo: Arc<dyn XxxRepository>,
    // 필요한 다른 서비스/usecase 를 Arc<dyn Trait> 로 주입
}

impl DefaultXxxUsecase {
    pub fn new(repo: Arc<dyn XxxRepository>, /* ... */) -> Self { ... }
}

#[async_trait::async_trait]
impl XxxUsecase for DefaultXxxUsecase { /* ... */ }
```

현재 조립 상세:

| Default | 주입 |
|---|---|
| `DefaultUserUsecase` | `Arc<dyn UserRepository>` + `Arc<dyn PasswordService>` |
| `DefaultLoginUsecase` | `Arc<dyn UserRepository>` + `Arc<dyn PasswordService>` |
| `DefaultRbacUsecase` | `Arc<dyn RbacRepository>` + 내부 `HashMap<i64, HashSet<String>>` 캐시 (Mutex/RwLock) |
| `DefaultJwtIdentUsecase` | `Arc<dyn LoginUsecase>` + `Arc<dyn UserUsecase>` + `Arc<dyn JwtService>` + `Arc<dyn JwtIdentRepository>` |

`DefaultJwtIdentUsecase` 는 trait-on-trait 조합 — 다른 usecase 를 호출하므로 조립 순서 주의 (`../crates/shinespark-app.md` 의 AppContainer 조립 단락).

## 경계 규칙

- **Usecase 는 SQL 을 모른다** — 모든 DB 접근은 repository 로 위임
- **Repository 는 비즈니스 로직을 모른다** — 검증/변환은 usecase 에서
- **DTO 는 trait 파일 안에** — 별도 dto 모듈 만들지 않음 (현재 관례)
- **에러 매핑은 `shinespark::Error` 단일 타입으로** — HTTP 매핑은 `ApiError` 에서 자동

## 추가 시 체크리스트

1. 새 method 는 `&mut Handle<'_>` 첫 인자 필수
2. 새 DTO 는 `Command` / `Query` suffix, `#[derive(Debug)]` 최소
3. `Default*Usecase::new` 시그니처도 함께 갱신 — `AppContainer::new` 조립이 깨지지 않도록
4. trait 변경 시 `MockUserRepository` 가 영향을 받는지 확인 (`UserRepository` 쪽만)
5. 캐시 (`DefaultRbacUsecase`) 를 건드렸다면 `load` / CRUD 의 캐시 동기화도 반영
