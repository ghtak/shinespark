# CLAUDE.md

0. Context First (No Code zones)
질의응답 우선: 요청을 받아도 즉시 코딩하지 않는다. 질문을 통해 비즈니스 요구사항, 제약 조건, 기술 스택을 먼저 확정한다.

설계 동기화: 파악된 내용을 바탕으로 개념 설계(Conceptual Design)를 제안하고, 사용자의 "진행합시다" 또는 **"플랜 짜주세요"**라는 명시적 승인이 있을 때만 다음 단계로 이동한다.

1. Plan-First, Code-Later
체크리스트 필수: 코딩 전 작업 계획을 마크다운 체크리스트(- [ ]) 형태로 제시한다.

구조 우선: 폴더 트리와 파일 레이아웃을 먼저 확정한다.

Atomic Task: 한 번의 응답에 하나의 논리적 단위(Atomic unit)만 처리한다. 작업 완료 시마다 컨펌을 구한다.

2. Development Phases
Phase 1 (Architecture): 디렉토리 구조 및 의존성 설계.

Phase 2 (Contract): 로직 구현 전 Interface, Type, Trait, Abstract Class 선언.

Phase 3 (Mock-up): 인터페이스 기반의 가동 가능한 최소 코드(Dummy data) 및 필요 시 Unit Test 작성.

Phase 4 (Implementation): Mock을 실제 비즈니스 로직, DB, API 연동 코드로 교체.

3. Communication & Token Efficiency
간결성: 장황한 설명은 지양하고 코드 주석이나 핵심 요약 위주로 소통한다.

변경 요약: 코드 수정/생성 후 변경된 핵심 사항을 3줄 이내로 요약 보고한다.

리스크 고지: 설계 결함이나 병목 예상 지점 발견 시 즉시 제언한다.

상태 추적: 체크리스트의 진행 상황을 매 응답마다 업데이트하여 컨텍스트를 유지한다.


This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build all crates
cargo build

# Run all tests (with output)
cargo test -- --nocapture

# Run a single test by name
cargo test --package shinespark --lib -- config::tests::test_load_env --exact --nocapture

# Run the application
cargo run --package shinespark-app

# Check compilation without building
cargo check

# Format code (rustfmt.toml configures block indent style, chain_width=100)
cargo fmt
```

Tests use `serial_test` for tests that mutate environment variables — use `#[serial]` when writing tests that touch env state.

Database integration tests are marked `#[ignore]` and require a live DB connection via `DATABASE_URL`. Run with `cargo test -- --ignored`.

## Project Structure

Three-crate Cargo workspace:

- **`shinespark`** — Core infrastructure library (database, config, crypto, tracing, HTTP utilities). No business logic.
- **`shinespark-identity`** — Identity domain crate (users, authentication, RBAC). Pure domain logic with trait-based use cases and repository interfaces.
- **`shinespark-app`** — Binary. Wires everything together: loads config, initializes tracing, creates DB pool, builds `AppContainer` (DI), seeds admin user, starts Axum HTTP server.

## Architecture: Clean/Hexagonal with DDD

Layers (inner to outer):

1. **Entities** (`shinespark-identity/src/entities.rs`) — `User`, `UserIdentity`, `UserAggregate`, value objects. No framework dependencies.
2. **Use Case Traits** (`shinespark-identity/src/usecases/`) — Async traits: `UserUsecase`, `LoginUsecase`, `RbacUsecase`. Define ports.
3. **Infrastructure Implementations** (`shinespark-identity/src/infra/`) — `DefaultUserUsecase`, `DefaultLoginUsecase`, `SqlxUserRepository`. Implement the traits.
4. **App Wiring** (`shinespark-app/src/main.rs`) — `AppContainer` holds `Arc<dyn Trait>` instances. Axum route handlers receive container via `State`.

Dependencies flow inward only: `shinespark-app` → `shinespark-identity` → `shinespark`.

`MockUserRepository` (`shinespark-identity/src/infra/mock_user_repository.rs`) is available for unit tests that don't need a real database.

## Configuration System

Config is loaded in two steps in `main.rs`:
1. `AppConfig::load_dotenv()` — loads `.env` and `configs/shinespark-<mode>.env`
2. `AppConfig::new()` — reads `configs/shinespark.toml` then overlays env vars

- Mode is controlled by `RUN_MODE` env var (`local`, `dev`, `production`)
- Env var overrides use `APP__` prefix with `__` as separator (e.g., `APP__DATABASE__URL`)
- In release mode, config files are searched relative to the executable directory

Key config sections: `database`, `http`, `trace`, `crypto`.

## Database Abstraction (`shinespark/src/db/`)

`Handle` is a three-variant type alias (`BasicHandle<Driver>`) that enables flexible query contexts:
- `Handle::Pool` — standard connection pool (default, via `db.handle()`)
- `Handle::Tx` — within an active transaction (via `db.tx().await`)
- `Handle::Conn` — single pooled connection (via `db.conn().await`)

Repositories accept `&mut Handle<'_>` and call `.inner()` to get the executor. Transactions are committed via `handle.commit().await`.

Multi-database support (PostgreSQL, SQLite, MySQL) via Cargo feature flags. Default is `db-driver-postgres`. Only one driver feature may be active at a time.

Database schema uses the `shs_iam_` table prefix. Migration SQL is in `shinespark-identity/sql/`.

### Query Building Traits (`shinespark/src/db.rs`)

- `SqlStatement` — wraps a `&'static str` SQL fragment; provides `.as_query_as::<O>()` and `.as_builder()` helpers.
- `SqlComposer` — implemented by query filter structs; `.compose(builder)` appends conditions to a `QueryBuilder`.
- `SqlBuilderExt` — extends `QueryBuilder` with `.push_option(sql, &Option<T>)` to conditionally append a bound parameter.

## HTTP Layer (`shinespark-app/src/http/`)

- **Extractors**: `CurrentUser` (requires valid session), `AdminUser` (requires admin role) — both are Axum extractors that reject unauthenticated/unauthorized requests automatically. **Note**: `AdminUser`'s RBAC check is currently commented out — it only verifies authentication, not the admin role.
- **Response types**: `ApiResponse<T>` and `ApiError` serialize to consistent JSON (`{ "code": "...", "data": ... }` / `{ "code": "...", "message": "..." }`). Domain errors map to HTTP status codes in `api_response.rs`.
- **Session**: `tower-sessions` with `MemoryStore`, 1-day inactivity expiry. Sessions are lost on process restart.

Current routes:
- `POST /identity/login`
- `POST /identity/logout`
- `GET /identity/me`

## Identity Domain Details

- `UserAggregate` = `User` + `Vec<UserIdentity>` + `role_ids: Vec<i64>`
- `UserStatus`: Active, Inactive, Pending, Suspended, Deleted
- `AuthProvider` (identity provider enum): Local, Google, Apple
- Audit logs are written for login attempts (`shs_iam_user_audit_log`)
- RBAC uses `shs_iam_permission`, `shs_iam_role`, `shs_iam_role_permission`, `shs_iam_user_role`

## Seed Data

On startup, `shinespark-identity/src/infra/seed_user.rs` seeds an admin user:
- Email: `admin@shinespark.dev`
- Password: `password`

This is a dev convenience — check before deploying.

## Password Hashing

`shinespark/src/crypto.rs` provides a `PasswordService` trait with an Argon2id implementation (`B64PasswordService`). It supports rehash detection (`needs_rehash`) so credentials can be upgraded transparently on login.
