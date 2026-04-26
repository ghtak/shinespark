---
title: 로그인 페이지 구현 (Cookie 기반 JWT, 투명 갱신)
status: draft
created: 2026-04-26
updated: 2026-04-26
---

## 목표

- `/auth/login` 에 로컬(이메일/비밀번호) + Google OAuth 로그인 UI 제공
- 토큰은 HttpOnly 쿠키로 발급 — 클라이언트는 인증 존재를 알 필요 없음
- 백엔드 미들웨어가 access_token 만료 시 refresh_token 으로 투명하게 갱신
- 로그인 후 `/` 로 이동

---

## 설계 결정

### 토큰 쿠키

| 쿠키명 | 옵션 |
|---|---|
| `access_token` | HttpOnly, SameSite=Lax, Path=/ |
| `refresh_token` | HttpOnly, SameSite=Lax, Path=/ |

- 두 쿠키 모두 `Path=/` — 경로 제한 없음
- `Secure`: dev `false`, prod `true` (RUN_MODE 분기)

### 투명 갱신 미들웨어 (`AuthMiddleware`)

클라이언트는 토큰 만료·갱신을 알지 못한다.  
보호 라우트에 미들웨어를 적용하고, 미들웨어가 다음 순서로 처리:

```
요청 수신
  ├─ access_token 유효 → 클레임을 Request Extension 에 삽입, 다음 핸들러 실행
  ├─ access_token 만료/없음 + refresh_token 유효
  │     → 새 TokenPair 발급
  │     → 클레임을 Extension 에 삽입
  │     → 핸들러 실행 후 응답에 Set-Cookie 추가 (새 토큰 쌍)
  └─ 둘 다 유효하지 않음 → Redirect /auth/login
```

### `CookieJwtUser` Extractor

- Request Extension 에서 미들웨어가 삽입한 `JwtClaims` 를 꺼내는 얇은 래퍼
- 미들웨어가 검증을 완료했으므로 extractor 는 verify 하지 않음

### 기존 API 유지

- `/identity/jwt/login`, `/identity/oauth2/{provider}/callback` — 그대로 유지
- 웹 UI 전용은 `/auth/*` 경로에만 추가

### 공개 라우트 (`/auth/login`, OAuth 콜백)

미들웨어 미적용. 인증 없이 접근 가능.

---

## 의존성 추가

`shinespark-app/Cargo.toml`:

```toml
axum-extra = { version = "0.10", features = ["cookie"] }
```

---

## 신규 파일

```
templates/
  auth/
    login.html                    ← 로그인 페이지 (로컬 폼 + Google 버튼)
shinespark-app/src/http/
  cookie_jwt.rs                   ← AuthMiddleware + CookieJwtUser extractor
routes.rs (수정)                  ← web::auth 서브모듈, 미들웨어 적용
```

---

## 라우트

### 공개 (미들웨어 미적용)

| Method | Path | 핸들러 |
|---|---|---|
| GET | `/auth/login` | `auth::login_page` |
| POST | `/auth/login` | `auth::login` — 쿠키 Set + Redirect `/` |
| GET | `/auth/oauth2/{provider}/callback` | `auth::oauth2_callback` — 쿠키 Set + Redirect `/` |

### 보호 (미들웨어 적용)

| Method | Path | 핸들러 |
|---|---|---|
| POST | `/auth/logout` | `auth::logout` — 쿠키 삭제 + Redirect `/auth/login` |

### 기존 web 라우트 보호

- `GET /` (index) 등 기존 `web::routes()` 에 `AuthMiddleware` layer 추가

---

## 미들웨어 설계 (`cookie_jwt.rs`)

```rust
pub async fn auth_middleware(
    State(container): State<Arc<AppContainer>>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Response {
    // 1. access_token 검증
    if let Some(claims) = try_verify_access(&container, &jar) {
        req.extensions_mut().insert(claims);
        return next.run(req).await;
    }

    // 2. refresh_token 으로 갱신 시도
    if let Some((claims, new_pair)) = try_refresh(&container, &jar).await {
        req.extensions_mut().insert(claims);
        let mut res = next.run(req).await;
        // 응답에 새 쿠키 추가
        set_token_cookies(&mut res, &new_pair, secure);
        return res;
    }

    // 3. 인증 불가 → 로그인으로
    Redirect::to("/auth/login").into_response()
}
```

---

## `CookieJwtUser` Extractor

```rust
pub struct CookieJwtUser(pub JwtClaims);

impl<S> FromRequestParts<S> for CookieJwtUser {
    // req.extensions().get::<JwtClaims>() — 미들웨어가 삽입한 값
    // None 이면 500 (미들웨어 없이 핸들러가 호출된 버그)
}
```

---

## 핸들러 설계

### `POST /auth/login`

```rust
async fn login(container, jar: CookieJar, Form(req)) -> (CookieJar, Redirect)
  jwt_usecase.login(LoginCommand::Local { email, password }) → TokenPair
  → set_token_cookies(jar, pair)
  → Redirect::to("/")
```

### `GET /auth/oauth2/{provider}/callback`

```rust
async fn oauth2_callback(container, jar, Path(provider), Query(params))
  → (CookieJar, Redirect)
  usecase.callback(...) → user
  jwt_usecase.login(LoginCommand::Social { ... }) → TokenPair
  → set_token_cookies(jar, pair)
  → Redirect::to("/")
```

### `POST /auth/logout`

```rust
async fn logout(jar: CookieJar) -> (CookieJar, Redirect)
  jar -= "access_token"
  jar -= "refresh_token"
  → Redirect::to("/auth/login")
```

### 공통 헬퍼

```rust
fn set_token_cookies(jar: CookieJar, pair: &TokenPair, secure: bool) -> CookieJar
  // access_token, refresh_token 모두 HttpOnly, SameSite=Lax, Path=/
```

---

## 템플릿: `templates/auth/login.html`

`base.html` 상속. Tailwind + Flowbite CDN.

- 로컬 로그인: `<form method="POST" action="/auth/login">`  
  email input, password input, 제출 버튼
- 구분선
- Google 로그인: `<a href="/identity/oauth2/google/login">` 버튼
- 에러: `{% if error %}`

> JS 불필요. 순수 HTML 폼.

---

## 설정 변경

`configs/shinespark-local.toml`:

```toml
[google_login]
redirect_uri = "http://localhost:8085/auth/oauth2/google/callback"
```

Google Console Authorized redirect URI 동일 URL 등록 필요.

---

## 체크리스트

- [ ] `shinespark-app/Cargo.toml` — `axum-extra` 추가
- [ ] `shinespark-app/src/http/cookie_jwt.rs` — `auth_middleware` + `CookieJwtUser` + `set_token_cookies` 구현
- [ ] `shinespark-app/src/http.rs` — `cookie_jwt` 모듈 등록
- [ ] `templates/auth/login.html` — 로그인 페이지 작성
- [ ] `routes.rs` — `web::auth` 서브모듈 (3개 공개 핸들러 + 1개 보호 핸들러)
- [ ] `routes.rs` — `web::routes()` 에 `auth::routes()` merge, 기존 `/` 에 `AuthMiddleware` layer 적용
- [ ] `configs/shinespark-local.toml` — `redirect_uri` 변경
- [ ] 브라우저 로컬 로그인 → 쿠키 확인 → `/` 이동
- [ ] access_token 만료 후 요청 → 투명 갱신 확인
- [ ] 로그아웃 → 쿠키 삭제 → `/auth/login` 이동
- [ ] Google 로그인 → 콜백 → 쿠키 확인 → `/` 이동

---

## 리스크

| 항목 | 내용 |
|---|---|
| CSRF | SameSite=Lax 로 기본 방어. POST 변경 요청에 CSRF 토큰 추가 고려 (범위 밖) |
| 미들웨어 미적용 라우트 에서 CookieJwtUser 사용 | Extension 누락 → 500. 라우트 구성 시 미들웨어 적용 여부 확인 필수 |
| Google Console redirect_uri | 미등록 시 OAuth 오류 |
| prod Secure 플래그 | HTTPS 없이 Secure=true 시 쿠키 미전송 |
