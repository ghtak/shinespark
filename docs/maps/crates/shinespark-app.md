---
scope: shinespark-app 바이너리 — 부팅 시퀀스, AppContainer, HTTP 와이어링
when-to-read: main.rs / DI / 라우트 등록 / session·jwt layer 를 건드릴 때
budget: 250
related:
  - ../_index.md
  - ../architecture.md
  - ../http-layer.md
  - ../crates/shinespark-core.md
  - ../crates/shinespark-identity.md
updated: 2026-04-18
---

## TL;DR

- 바이너리 진입점: `shinespark-app/src/main.rs`
- DI 컨테이너: `AppContainer` — `Arc<dyn Trait>` 필드로 모든 usecase/service 보관
- HTTP 레이아웃: `src/http/` 에 `routes.rs` / `api_response.rs` / `session.rs` / `jwt.rs`
- 라우트 그룹 2개: `identity::session` (tower-sessions) / `identity::jwt` (Bearer)
- session 은 MemoryStore — 재시작 시 세션 소실 (dev 용)

## 모듈 트리

```
shinespark-app/src/
  main.rs                # 부팅 + AppContainer 정의
  http.rs                # http 모듈 재노출
  http/
    routes.rs            # identity::session / identity::jwt 서브모듈 + 핸들러
    api_response.rs      # ApiResponse<T>, ApiError, ApiResult<T> + 에러 매핑
    session.rs           # CurrentUser extractor + tower-sessions 레이어
    jwt.rs               # JwtUser extractor + Bearer 파싱/검증
```

상세 동작은 `../http-layer.md`.

## 부팅 시퀀스 (`main.rs`)

```rust
#[tokio::main] async fn main() {
    AppConfig::load_dotenv();              // 1. .env + configs/*.env
    let cfg = AppConfig::new()?;           // 2. toml + env overlay
    shinespark::trace::init(&cfg.trace)?;  // 3. tracing
    let db = Database::new(&cfg.database)?;// 4. sqlx pool
    let container = Arc::new(
        AppContainer::new(db, &cfg)        // 5. 모든 Arc<dyn Trait> 조립
    );
    container.rbac_usecase
        .load(&mut container.db.handle()); // 6. RBAC 캐시 워밍
    seed_admin(&container)?;               // 7. 개발용 admin
    let router = Router::new()
        .route("/", ...)                   // 헬스체크 (DB ping)
        .merge(identity::routes())         // 8. 라우트 + session layer
        .with_state(container);
    shinespark::http::run(router, &cfg.http)? // 9. 바인드 + 서빙
}
```

## AppContainer

`main.rs:AppContainer` — 모든 도메인 의존성을 `Arc<dyn Trait>` 으로 보관하여 handler 에 `State<Arc<AppContainer>>` 로 주입.

필드:

| 필드 | 타입 | 출처 |
|---|---|---|
| `db` | `Database` | `shinespark::db::Database` |
| `user_usecase` | `Arc<dyn UserUsecase>` | `DefaultUserUsecase` |
| `login_usecase` | `Arc<dyn LoginUsecase>` | `DefaultLoginUsecase` |
| `rbac_usecase` | `Arc<dyn RbacUsecase>` | `DefaultRbacUsecase` |
| `jwt_ident_usecase` | `Arc<dyn JwtIdentUsecase>` | `DefaultJwtIdentUsecase` |
| `jwt_service` | `Arc<dyn JwtService>` | `HS256JwtService` |

`AppContainer::new(db, &cfg)` 내부에서 조립 순서 (대략):

1. `B64PasswordService` → `Arc<dyn PasswordService>`
2. `SqlxUserRepository` → `Arc<dyn UserRepository>`
3. `DefaultUserUsecase` (repo + password) → `Arc<dyn UserUsecase>`
4. `DefaultLoginUsecase` (repo + password) → `Arc<dyn LoginUsecase>`
5. `SqlxRbacRepository` → `DefaultRbacUsecase` (+ 내부 캐시) → `Arc<dyn RbacUsecase>`
6. `HS256JwtService` (JwtConfig) → `Arc<dyn JwtService>`
7. `SqlxJwtIdentRepository` → `DefaultJwtIdentUsecase` (login + user + jwt_service + repo) → `Arc<dyn JwtIdentUsecase>`

새 usecase 를 추가할 때는 이 조립 순서에 끼워 넣는다 (`../feature-lifecycle.md`).

## HTTP 와이어링

라우터 정의는 `http/routes.rs` 의 `identity::routes()` 가 반환하는 `Router<Arc<AppContainer>>`. session layer 는 `http::session::simple_layer()` 가 반환 (MemoryStore 기반 `SessionManagerLayer`).

현재 라우트 (세부는 `../http-layer.md`):

- **Session**: `POST /identity/session/login`, `POST /identity/session/logout`, `GET /identity/session/me`
- **JWT**: `POST /identity/jwt/login`, `POST /identity/jwt/logout`, `POST /identity/jwt/refresh`, `GET /identity/jwt/me`

Extractor:

- `CurrentUser` — session 에서 `UserAggregate` 복원, 실패 시 401
- `JwtUser` — Bearer → `JwtClaims` 검증, 실패 시 401

## Config 사용

`main.rs` 는 `AppConfig` 를 그대로 `AppContainer::new` 에 넘긴다. container 내부에서 필요한 섹션 (`jwt`, `crypto`) 만 꺼내 각 서비스 생성자에 전달.

`configs/` 및 env overlay 규약: `../crates/shinespark-core.md` 의 AppConfig 섹션 참고.

## 핸들러 작성 관례

- `State(container): State<Arc<AppContainer>>` 로 주입 받기
- 인증 필요 시 두 번째 arg 에 `CurrentUser` 또는 `JwtUser`
- 요청 DTO: `Json<Req>`, 응답: `ApiResult<Resp>` (= `Result<ApiResponse<Resp>, ApiError>`)
- `container.xxx_usecase.method(&mut container.db.handle(), ...).await?` — `?` 가 `shinespark::Error` → `ApiError` 자동 변환

## 테스트

- `shinespark-app` 자체는 조립 중심이라 단위 테스트 드묾
- identity 도메인 로직은 identity crate 의 `MockUserRepository` + `Default*Usecase` 로 테스트
- HTTP 통합 테스트는 `tokio::test` + `axum::serve` + `reqwest` 패턴 (필요 시 추가)
