---
scope: 모든 map 의 엔트리 — 어디에 무엇이 있는지 알려주는 라우팅 테이블
when-to-read: 세션 시작 시 항상 가장 먼저
budget: 150
related:
  - ./CONVENTIONS.md
  - ./MAINTENANCE.md
updated: 2026-04-18
---

## TL;DR

- 3-crate Cargo workspace: `shinespark` (infra) → `shinespark-identity` (domain) → `shinespark-app` (binary)
- 의존은 **안쪽으로만** 흐른다. `shinespark-app` → `shinespark-identity` → `shinespark`
- 작업 영역이 좁혀지면 아래 "Where to go" 표에서 해당 map 만 추가로 읽는다
- map 을 스스로 고치지 않는다 — 사용자가 요청할 때만 갱신 (`MAINTENANCE.md`)
- 심볼/경로는 `file.rs:Symbol` 링크로만. 코드 복붙 금지 (`CONVENTIONS.md`)

## 프로젝트 한 줄

Rust + Axum + sqlx 기반 IAM 백엔드. 클린/육각형 아키텍처 + DDD. 세 crate 워크스페이스로 infra / domain / binary 를 분리한다.

## Where to go

| Map | When-to-read | Path | Budget |
|---|---|---|---|
| Architecture | crate 경계, 의존 방향, 부팅 순서 결정 | `./architecture.md` | 200 |
| Feature lifecycle | 새 기능을 end-to-end 추가 | `./feature-lifecycle.md` | 200 |
| Crate: shinespark-core | db/config/crypto/trace/http/error/util 작업 | `./crates/shinespark-core.md` | 250 |
| Crate: shinespark-identity | identity 도메인 작업 진입 | `./crates/shinespark-identity.md` | 250 |
| Crate: shinespark-app | 부팅·DI·HTTP 와이어링 | `./crates/shinespark-app.md` | 250 |
| Domain: usecases | usecase trait / `Default*Usecase` / DTO 추가·수정 | `./domain/identity-usecases.md` | 300 |
| Domain: repositories | repository trait / `Sqlx*` / `Mock*` 수정 | `./domain/identity-repositories.md` | 250 |
| Domain: SQL | `sql/` 파일, 스키마, 테이블 추가·수정 | `./domain/identity-sql.md` | 200 |
| HTTP layer | 라우트·extractor·응답 포맷 수정 | `./http-layer.md` | 250 |
| Conventions | map 파일을 새로 만들거나 수정 | `./CONVENTIONS.md` | 100 |
| Maintenance | map 갱신 요청을 받았을 때 | `./MAINTENANCE.md` | 120 |

## 전역 불변식

다른 map 들은 아래 사실을 **다시 기술하지 않고** 이 섹션을 참조한다.

- **Workspace 구성**: `shinespark`, `shinespark-identity`, `shinespark-app`
- **의존 방향**: `app → identity → core` (역방향 금지)
- **Layering**: entity → usecase trait → infra impl (`Default*`, `Sqlx*`) → app 조립 (`AppContainer`)
- **DB 테이블 prefix**: `shs_iam_*` (identity 도메인 전용 네임스페이스)
- **Permission code 포맷**: `Resource.action.scope`, 와일드카드 `*.*.all`
- **DB feature flags**: `db-driver-postgres` (기본), `db-driver-sqlite`, `db-driver-mysql` — **상호 배타**, 동시에 하나만 활성
- **`Handle` 3-variant**: `Handle::Pool` / `Handle::Tx` / `Handle::Conn` — repository 는 `&mut Handle<'_>` 수신, `.inner()` 로 executor 획득
- **Repository 경계**: SQL 실행은 repository, 비즈니스 로직은 usecase (`Default*Usecase` 내부)
- **Config 계층**: `configs/shinespark.toml` + `configs/shinespark-<RUN_MODE>.env` + env var (`APP__<section>__<key>`) 순으로 overlay
- **Error 허브**: `shinespark::Error` (단일 enum) → `ApiError` (HTTP 매핑은 `http-layer.md`)
- **Auth 경로 2종**: Session (`tower-sessions` MemoryStore) / JWT (HS256, access+refresh). extractor: `CurrentUser` / `JwtUser`
- **Seed admin**: `admin@shinespark.dev` / `password` — 부팅 시 `seed_admin` 이 생성 (dev 용)

## 소스 진입점

- 워크스페이스 루트: `C:\Users\tlab\works\shinespark\Cargo.toml`
- 바이너리 main: `shinespark-app/src/main.rs`
- 프로젝트 규칙 (map 이 아닌): 루트 `CLAUDE.md`
- 도메인/설정/마이그레이션 가이드: `docs/01. setup.md`, `docs/02. config.md`, `docs/03. migrations.md`, `docs/04. rbac.md`
- 설계·결정 기록: `docs/plan/` (이 map 과 분리된 계층)

## 갱신

이 파일과 모든 하위 map 은 **사용자 요청이 있을 때만** 갱신한다. 자세한 절차와 트리거 매트릭스는 `./MAINTENANCE.md`.
