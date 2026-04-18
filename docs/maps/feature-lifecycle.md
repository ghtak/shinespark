---
scope: 새 기능을 entity 부터 HTTP 까지 end-to-end 추가하는 레시피
when-to-read: 새 도메인 기능/유스케이스/라우트를 추가할 때
budget: 200
related:
  - ./_index.md
  - ./architecture.md
  - ./domain/identity-usecases.md
  - ./domain/identity-repositories.md
  - ./domain/identity-sql.md
  - ./http-layer.md
updated: 2026-04-18
---

## TL;DR

- 순서: **entity → usecase trait → repository trait → `Default*Usecase` → `Sqlx*Repository` → SQL 파일 → HTTP 라우트 → `AppContainer` 조립**
- 인터페이스 먼저, 구현 나중 (CLAUDE.md 의 Phase 2 Contract 규칙)
- DTO 는 usecase 파일 안에 `<Action>Command` / `<Query>Query` 구조체로
- repository 는 `&mut Handle<'_>` 만 받음. `.inner()` 로 sqlx executor 획득
- 에러는 `shinespark::Error` 로 통일, HTTP 매핑은 `ApiResponse`/`ApiError` 에서 자동

## 작업 순서 (체크리스트)

```
- [ ] 1. Entity / Value object 정의 (필요 시)
- [ ] 2. Usecase trait + Command/Query DTO 선언
- [ ] 3. Repository trait 선언 (DB 접근 최소 단위)
- [ ] 4. `DefaultXxxUsecase` 구현 (비즈니스 로직)
- [ ] 5. `SqlxXxxRepository` 구현 + SQL 파일 추가
- [ ] 6. (선택) `MockXxxRepository` — 테스트용
- [ ] 7. HTTP 라우트 + handler
- [ ] 8. `AppContainer` 에 필드 추가 + 조립
- [ ] 9. 라우터에 merge
- [ ] 10. 테스트 (unit with Mock, integration with #[ignore])
```

## 단계별 상세

### 1. Entity

`shinespark-identity/src/entities.rs` 에 추가. 값 객체는 enum 또는 newtype 으로. framework 의존 금지, serde derive 는 허용.

### 2. Usecase trait

`shinespark-identity/src/usecases/<feature>_usecase.rs` 신규:

```rust
#[async_trait]
pub trait XxxUsecase: Send + Sync {
    async fn do_something(
        &self,
        handle: &mut Handle<'_>,
        cmd: DoSomethingCommand,
    ) -> Result<ReturnType>;
}

pub struct DoSomethingCommand { /* ... */ }
```

- DTO 는 trait 파일 안에 같이 둔다 (Command/Query 네이밍)
- `handle` 은 항상 첫 인자 (트랜잭션 컨텍스트 주입)
- 동일 crate `src/usecases/mod.rs` 에서 `pub use` 재노출

### 3. Repository trait

`shinespark-identity/src/repositories/<feature>_repository.rs` 신규. usecase 가 필요로 하는 **최소 단위** DB 연산만 노출한다. 복합 로직은 usecase 에서 조합.

```rust
#[async_trait]
pub trait XxxRepository: Send + Sync {
    async fn find_by_id(&self, handle: &mut Handle<'_>, id: i64) -> Result<Option<Xxx>>;
}
```

### 4. `Default*Usecase`

`shinespark-identity/src/infra/default_<feature>_usecase.rs`:

- struct 필드는 `Arc<dyn XxxRepository>` + 필요한 서비스 (`Arc<dyn PasswordService>` 등)
- `#[async_trait] impl XxxUsecase for DefaultXxxUsecase` 로 구현
- 에러 매핑은 `shinespark::Error` 로 통일
- 비즈니스 로직 (검증, 트랜잭션 경계 선언, 여러 repo 호출 조합) 은 여기

### 5. `Sqlx*Repository` + SQL

`shinespark-identity/src/infra/sqlx_<feature>_repository.rs`:

- SQL 본문은 `shinespark-identity/sql/<feature>_repository/*.sql` 파일로 분리
- `sqlx_statement.rs` 의 enum 에 variant 추가 (`include_str!` 사용)
- 메서드 본문에서 `handle.inner()` 로 executor 획득 후 `.fetch_*` / `.execute`
- SQL 규약은 `./domain/identity-sql.md` 참조 (table prefix `shs_iam_*`)

### 6. Mock (선택)

`shinespark-identity/src/infra/mock_<feature>_repository.rs`:

- `Mutex<HashMap<...>>` 기반 in-memory
- 단위 테스트에서 `Arc<dyn XxxRepository>` 자리에 주입
- 기존 예시: `mock_user_repository.rs`

### 7. HTTP 라우트

`shinespark-app/src/http/routes.rs`:

- `identity::session` 또는 `identity::jwt` 서브모듈 중 적절한 곳, 또는 신규 모듈 생성
- handler 시그니처:
  ```rust
  async fn handler(
      State(container): State<Arc<AppContainer>>,
      user: CurrentUser,        // 또는 JwtUser — 인증이 필요하면
      Json(req): Json<DtoReq>,
  ) -> ApiResult<DtoResp> { ... }
  ```
- `?` 로 `shinespark::Error` 를 `ApiError` 로 자동 변환
- 응답 DTO 는 `Serialize` derive, `ApiResponse<T>` 로 wrap

### 8~9. `AppContainer` + 라우터

`shinespark-app/src/main.rs`:

- `AppContainer` 구조체에 `pub xxx_usecase: Arc<dyn XxxUsecase>` 필드 추가
- `AppContainer::new` 내부에서 repository/usecase 를 조립
- `main()` 의 라우터 정의에 `.merge(routes())` 추가

### 10. 테스트

- 단위: `MockXxxRepository` 주입, `#[tokio::test]`
- env 건드리면 `#[serial]`
- 실 DB 필요하면 `#[ignore]` + `cargo test -- --ignored`

## 금지 / 주의

- usecase trait 에서 sqlx 타입 노출 금지 (도메인 순수성)
- repository 에서 비즈니스 규칙 검증 금지 (단순 CRUD 만)
- `AppContainer` 없이 handler 에서 직접 DB 풀 참조 금지
- SQL inline 금지 — 반드시 `sql/` 디렉터리의 파일로

## 참고 예시

- 가장 단순한 흐름: `UserUsecase` → `DefaultUserUsecase` → `SqlxUserRepository` → `sql/user_repository/`
- 여러 repo 오케스트레이션: `DefaultJwtIdentUsecase` 가 `LoginUsecase` + `UserUsecase` + `JwtService` + `JwtIdentRepository` 조합
