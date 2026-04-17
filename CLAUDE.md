# CLAUDE.md

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
