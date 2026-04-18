---
scope: shinespark (코어 인프라) crate 의 모듈 지도와 공개 인터페이스
when-to-read: db / config / crypto / trace / http / error / util 를 수정하거나 infra trait 을 새로 정의할 때
budget: 250
related:
  - ../_index.md
  - ../architecture.md
  - ../domain/identity-repositories.md
updated: 2026-04-18
---

## TL;DR

- 비즈니스 로직 없음 — 도메인 crate 들이 의존하는 **infra 베이스**
- 공개 trait: `PasswordService`, `SqlStatement`, `SqlComposer`, `SqlBuilderExt`
- 공개 struct: `Database`, `Handle<'c>` (Pool/Tx/Conn 3-variant), `AppConfig`
- DB driver 는 feature flag 로 택일 (**상호 배타**)
- 에러 허브: `shinespark::Error` (단일 enum) + `Result<T>` alias

## 범위 / 비범위

- ✅ DB 커넥션/트랜잭션 추상화, config loader, password hashing, tracing 부팅, error 통합, HTTP 서버 런너
- ❌ 도메인 타입, SQL 문, 라우트, 세션, JWT 로직 (모두 identity / app 소관)

## 모듈 트리

| 파일 | 역할 |
|---|---|
| `shinespark/src/lib.rs` | crate root. config/crypto/db/error/http/trace/util 재노출 |
| `shinespark/src/config.rs` | 계층적 config loader (TOML + env overlay) |
| `shinespark/src/crypto.rs` | `PasswordService` trait + 3종 impl (Argon2/PBKDF2/B64) |
| `shinespark/src/db.rs` | DB 공개 API (`Database`, `Handle`), SQL trait 군 |
| `shinespark/src/db/handle.rs` | `BasicHandle<DB>` 본체 + executor 어댑터 |
| `shinespark/src/error.rs` | `Error` enum + `Result<T>` + 코드 문자열 매핑 |
| `shinespark/src/http.rs` | `run(router, &HttpConfig)` — Axum 바인드/서빙 |
| `shinespark/src/trace.rs` | console/file tracing 초기화 |
| `shinespark/src/util.rs` | 경로 탐색 유틸 (workspace_root, base_path 등) |

## 공개 trait

### `PasswordService`
`shinespark/src/crypto.rs:PasswordService`

```
hash(password)      -> Result<String>
verify(pw, hash)    -> Result<bool>
needs_rehash(hash)  -> bool
```

stock impl: `Argon2PasswordService`, `Pbkdf2PasswordService`, `B64PasswordService` (dev). `Argon2Config` 는 `AppConfig.crypto`.

### `SqlStatement`
`shinespark/src/db.rs:SqlStatement`

static SQL fragment wrapper. `as_query_as::<O>()`, `as_builder()` 헬퍼 제공. `include_str!` 로 읽은 SQL 을 감싸는 용도.

### `SqlComposer`
`shinespark/src/db.rs:SqlComposer`

필터/쿼리 빌더 타입이 구현. `.compose(builder)` 로 `QueryBuilder` 에 WHERE 조건을 덧붙임. identity crate 의 조회 DTO 가 이 trait 을 구현해 동적 쿼리를 만든다.

### `SqlBuilderExt`
`shinespark/src/db.rs:SqlBuilderExt`

`QueryBuilder` 확장: `.push_option(sql, &Option<T>)` — `Some` 일 때만 조건 덧붙이기.

## 공개 struct

### `Database`
`shinespark/src/db.rs:Database`

sqlx pool wrapper. 팩토리:

- `Database::new(&DatabaseConfig)` — 생성
- `.handle()` — `Handle::Pool`
- `.tx().await` — `Handle::Tx` (새 트랜잭션)
- `.conn().await` — `Handle::Conn` (단일 커넥션 획득)

### `Handle<'c>`
`shinespark/src/db/handle.rs:BasicHandle` (type alias `Handle<'c> = BasicHandle<'c, Driver>`)

3-variant enum:

| Variant | 용도 |
|---|---|
| `Handle::Pool` | 기본. auto-commit 쿼리 |
| `Handle::Tx` | 트랜잭션 내부 |
| `Handle::Conn` | 커넥션 고정이 필요한 경우 |

공통 메서드: `.inner()` (sqlx executor 반환), `.begin()`, `.commit()`, `.rollback()`.

repository 는 `&mut Handle<'_>` 수신 → `.inner()` 로 executor 획득 → `sqlx::query*` 실행.

### `AppConfig`
`shinespark/src/config.rs:AppConfig`

섹션: `database`, `http`, `trace`, `crypto`, `jwt` 등. 로딩 단계:

1. `AppConfig::load_dotenv()` — `.env`, `configs/shinespark-<mode>.env` 읽기
2. `AppConfig::new()` — `configs/shinespark.toml` + env var overlay

env 네이밍: `APP__<section>__<key>` (구분자 `__`). 예: `APP__DATABASE__URL`.

관련 문서: `docs/02. config.md`.

## Error

`shinespark/src/error.rs:Error`

| Variant | 코드 |
|---|---|
| `Internal(anyhow::Error)` | `INTERNAL` |
| `IllegalState(Cow<str>)` | `ILLEGAL_STATE` |
| `NotImplemented` | `NOT_IMPLEMENTED` |
| `UnAuthorized` | `UNAUTHORIZED` |
| `DatabaseError(anyhow::Error)` | `DATABASE_ERROR` |
| `NotFound` | `NOT_FOUND` |
| `AlreadyExists` | `ALREADY_EXISTS` |
| `InvalidCredentials` | `INVALID_CREDENTIALS` |

type alias: `shinespark::Result<T> = Result<T, Error>`. HTTP 매핑은 `../http-layer.md`.

## Feature flags

| Flag | 기본 | `type Driver = ...` |
|---|---|---|
| `db-driver-postgres` | ✅ | `sqlx::Postgres` |
| `db-driver-sqlite` |  | `sqlx::Sqlite` |
| `db-driver-mysql` |  | `sqlx::MySql` |

컴파일 타임에 단일 driver 만 허용 (compile-time assertion).

## Extension points

| 확장하고 싶은 것 | 구현할 trait |
|---|---|
| 다른 해시 알고리즘 | `PasswordService` |
| 새 정적 SQL 템플릿 enum | `SqlStatement` |
| 동적 조건 조립 DTO | `SqlComposer` |
| `QueryBuilder` 유틸 | `SqlBuilderExt` (이미 있지만 메서드 추가 가능) |

## 내부 의존

| 모듈 | 의존 |
|---|---|
| `config` | (없음) |
| `crypto` | `config` (Argon2Config) |
| `db` | `config` (DatabaseConfig), `error` |
| `http` | `config` (HttpConfig) |
| `trace` | `config` (TraceConfig) |
| `error` | (없음) — 허브 |
| `util` | (없음) |
