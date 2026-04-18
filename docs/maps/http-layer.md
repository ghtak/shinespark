---
scope: shinespark-app HTTP 레이어 — 라우트, extractor, 응답/에러 포맷
when-to-read: 라우트 추가·수정, 인증 extractor 변경, ApiResponse/ApiError 포맷 수정
budget: 250
related:
  - ./_index.md
  - ./crates/shinespark-app.md
  - ./crates/shinespark-core.md
updated: 2026-04-18
---

## TL;DR

- 라우트 2그룹: **session** (tower-sessions) / **jwt** (Bearer)
- Extractor: `CurrentUser` (session) / `JwtUser` (JWT claims)
- 응답: `ApiResponse<T>` → `{ "code": "Ok", "data": T }`
- 에러: `ApiError` → `{ "code": "...", "message": "..." }` + HTTP status
- `shinespark::Error` → `ApiError` 자동 변환 (`From` impl, `?` 로 전파)
- Session store 는 `MemoryStore` — 재시작 시 세션 소실 (dev 용)

## 파일 배치

```
shinespark-app/src/http/
  routes.rs         # identity::session / identity::jwt 서브모듈
  api_response.rs   # ApiResponse<T> / ApiError / ApiResult<T>
  session.rs        # CurrentUser extractor, Session, session layer
  jwt.rs            # JwtUser extractor (Bearer token)
```

## 현재 라우트

모든 라우트의 state 는 `State<Arc<AppContainer>>`.

### Session 그룹 (`/identity/session`)

| Method | Path | Handler | 특징 |
|---|---|---|---|
| POST | `/identity/session/login` | Local 로그인 + 세션에 `UserAggregate` 저장 | Body: `{ email, password }` |
| POST | `/identity/session/logout` | `CurrentUser` 필요, 세션 키 제거 | 빈 body |
| GET | `/identity/session/me` | `CurrentUser` 필요, 세션 사용자 반환 | — |

### JWT 그룹 (`/identity/jwt`)

| Method | Path | Handler | 특징 |
|---|---|---|---|
| POST | `/identity/jwt/login` | 토큰 페어 발급 (`access_token`, `refresh_token`) | Body: `{ email, password }` |
| POST | `/identity/jwt/logout` | `JwtUser` 필요, refresh 무효화 | access token 은 만료까지 유효 |
| POST | `/identity/jwt/refresh` | refresh → 새 access+refresh 쌍 | Body: `{ refresh_token }` |
| GET | `/identity/jwt/me` | `JwtUser` 필요, `JwtClaims` 반환 | — |

세부 구현: `shinespark-app/src/http/routes.rs`.

## Extractor

### `CurrentUser`
파일: `shinespark-app/src/http/session.rs:CurrentUser`

- session 에서 `USER_SESSION_KEY` 에 저장된 `UserAggregate` 를 복원
- 실패 시 `ApiError(UNAUTHORIZED, 401)` 로 자동 변환
- 내부적으로 `tower_sessions::Session` 사용
- `OptionalFromRequestParts` + `FromRequestParts` 둘 다 구현됨

Handler 시그니처 예:

```rust
async fn handler(user: CurrentUser) -> ApiResult<UserAggregate> {
    Ok(ApiResponse::new(user.0))
}
```

### `JwtUser`
파일: `shinespark-app/src/http/jwt.rs:JwtUser`

- `Authorization: Bearer <token>` 헤더 파싱
- `container.jwt_service.verify(...)` 로 검증 → `JwtClaims` 반환
- 실패 (헤더 누락, 파싱 실패, 만료, 서명 불일치) 시 `ApiError(UNAUTHORIZED, 401)`
- `JwtClaims` 에는 `sub` (user_uid) 등 포함 — `logout` 등에서 사용

## 응답 / 에러 포맷

### `ApiResponse<T>`
파일: `shinespark-app/src/http/api_response.rs:ApiResponse`

```json
{
  "code": "Ok",
  "data": <T>
}
```

- 항상 HTTP 200 으로 직렬화
- `ApiResponse::new(data)` 로 생성
- `T: Serialize` 필요

### `ApiError`

```json
{
  "code": "<error code>",
  "message": "<display string>"
}
```

- `status_code` 는 body 에 포함되지 않고 HTTP status 로만 전달
- `code` 는 `shinespark::Error::code()` 에서 가져온 고정 문자열

### `ApiResult<T>`

```rust
pub type ApiResult<T> = Result<ApiResponse<T>, ApiError>;
```

Handler 표준 반환 타입. `?` 로 `shinespark::Error` 가 자동으로 `ApiError` 로 변환됨.

### 에러 매핑 테이블

`From<shinespark::Error> for ApiError` (`api_response.rs`):

| `shinespark::Error` variant | HTTP status | code |
|---|---|---|
| `Internal` / `DatabaseError` / `IllegalState` / `NotImplemented` | 500 | `INTERNAL` / `DATABASE_ERROR` / `ILLEGAL_STATE` / `NOT_IMPLEMENTED` |
| `NotFound` / `AlreadyExists` / `InvalidCredentials` | 400 | `NOT_FOUND` / `ALREADY_EXISTS` / `INVALID_CREDENTIALS` |
| `UnAuthorized` | 401 | `UNAUTHORIZED` |

## Session layer

- `shinespark-app/src/http/session.rs` 에 session 관련 부팅 유틸
- `MemoryStore` (in-memory) 기반, cookie name 기본 `SID`
- 기본 만료: 1일 inactivity
- `with_secure(false)` — 개발용 (프로덕션에서는 `true` + HTTPS 필요)
- 상수: `USER_SESSION_KEY` — 세션에 `UserAggregate` 를 저장할 때 쓰는 키

## 새 엔드포인트 추가

순서 (자세한 end-to-end 는 `./feature-lifecycle.md`):

1. `routes.rs` 의 적절한 서브모듈 (또는 신규 모듈) 에 handler 작성
2. 시그니처: `State<Arc<AppContainer>>` + (선택) `CurrentUser` / `JwtUser` + `Json<Req>` → `ApiResult<Resp>`
3. DTO 는 `#[derive(Serialize/Deserialize)]` — session 그룹은 `routes::identity::dto` 하위에, jwt 그룹은 모듈 내 local
4. 해당 모듈의 `pub fn routes() -> Router<Arc<AppContainer>>` 에 `.route(...)` 추가
5. 상위 `identity::routes()` 가 자동 merge

## 알림

- `CurrentUser` 는 인증만, **권한 검사는 하지 않음** — `container.rbac_usecase.check_perm(...)` 를 handler 에서 명시 호출해야 함
- `JwtUser` 역시 권한 체크 없음 — 토큰 유효성만 검증
- 세션 데이터는 `MemoryStore` 라 프로세스 재시작 시 모든 사용자 로그아웃됨 — 배포 전 persistent store 로 교체 고려
