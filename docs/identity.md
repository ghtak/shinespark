# Identity & RBAC System Design (Detailed)

이 문서는 `shinespark` 프로젝트의 Identity 및 RBAC 체계를 원자적 작업 단위(Atomic Tasks)로 나누어 상세히 기술합니다.

## 1. 전역 원칙
- **작은 작업 단위**: 한 번의 PR/Merge 단위로 완결성을 갖는 최소 기능을 구현합니다.
- **Contract First**: 로직 구현 전 Interface(Trait)를 먼저 확정합니다.

---

## 2. 데이터베이스 스키마 상세

### Phase 1: RBAC Core Tables
| 테이블 | 필드 | 설명 |
| :--- | :--- | :--- |
| `roles` | `id(INT, PK)`, `name(VARCHAR, UNIQUE)` | 역할 정의 (ADMIN, USER 등) |
| `permissions` | `id(INT, PK)`, `code(VARCHAR, UNIQUE)` | 권한 코드 (`user:create` 등) |
| `user_roles` | `user_id(FK)`, `role_id(FK)` | 사용자-역할 매핑 (M:N, PK:Composite) |
| `role_permissions` | `role_id(FK)`, `permission_id(FK)` | 역할-권한 매핑 (M:N, PK:Composite) |

### Phase 2: User & Identity (기존 migration 보완)
- `users`: `id`, `uid(UUID)`, `name`, `email`
- `user_identities`: `provider`, `provider_user_id`, `credential_hash` (Local용)

---

## 3. 크레이트 구조 상세 (`shinespark-identity`)

```text
shinespark-identity/
├── src/
│   ├── lib.rs          # 모듈 노출 및 공통 에러 정의
│   ├── entity/         # 도메인 모델 (Plain Structs)
│   │   ├── mod.rs
│   │   ├── user.rs     # User, Role, Permission 구조체
│   │   └── identity.rs # Auth Provider 정보
│   ├── repository/     # Persistence Interface (Traits)
│   │   ├── mod.rs
│   │   ├── user.rs     # UserRepository (CRUD)
│   │   └── rbac.rs     # RbacRepository (Permission Fetching)
│   ├── service/        # Business Logic
│   │   ├── mod.rs
│   │   ├── auth.rs     # Login, Register, Logout
│   │   └── access.rs   # Permission matching & Check logic
│   └── infra/          # SQLx Implementation (Optional, or in services)
```

---

## 4. 권한 체크 패턴

### Permission Code Naming
- `resource:action` 또는 `resource:sub-resource:action`
- 예: `user:view`, `post:edit`, `system:config:write`

### 코드 내 사용 예시
```rust
// Interface
trait AccessControl {
    fn has_permission(&self, permission: &str) -> bool;
}

// Usage in Application Service
pub async fn delete_user(user: CurrentUser, target_id: u64) -> Result<()> {
    if !user.has_permission("user:delete") {
        return Err(Error::PermissionDenied);
    }
    // ... logic
}
```

---

## 5. 단계별 작업 로드맵 (Atomic Tasks)

### Phase 1: Infrastructure & Core Schema
- [ ] 1.1 RBAC 기초 테이블 Migration 생성 (`roles`, `permissions`)
- [ ] 1.2 매핑 테이블 Migration 생성 (`user_roles`, `role_permissions`)
- [ ] 1.3 `users` 테이블 구조 조정 (기존 `nickname` -> `name` 등)

### Phase 2: Entity & Interface
- [ ] 2.1 도메인 엔티티 정의 (`User`, `Role`, `Permission` structs)
- [ ] 2.2 Repository Trait 정의 (`UserRepository`, `RbacRepository`)

### Phase 3: Auth & Identity implementation
- [ ] 3.1 로컬 계정 생성(Register) 서비스 구현 (Argon2 해싱 적용)
- [ ] 3.2 로컬 로그인(Login) 및 JWT 발급 기초 구현
- [ ] 3.3 JWT 검증 미들웨어/컴포넌트 구현

### Phase 4: RBAC Integration
- [ ] 4.1 로그인 시 사용자의 Permission 리스트를 함께 로드하는 로직 구현
- [ ] 4.2 권한 체크 Utility/Service 구현
