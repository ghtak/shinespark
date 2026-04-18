---
scope: 워크스페이스 구조, crate 경계, 의존 방향, 부팅 순서
when-to-read: crate 경계/의존성/부팅 초기화와 관련된 결정이 필요할 때
budget: 200
related:
  - ./_index.md
  - ./feature-lifecycle.md
  - ./crates/shinespark-core.md
  - ./crates/shinespark-identity.md
  - ./crates/shinespark-app.md
updated: 2026-04-18
---

## TL;DR

- 3-crate 워크스페이스: `shinespark` → `shinespark-identity` → `shinespark-app`
- 의존 방향은 단방향 (역참조 금지)
- `shinespark` 은 순수 infra — 비즈니스 로직 없음
- `shinespark-identity` 은 순수 도메인 — 프레임워크 의존 없음
- `shinespark-app` 만 Axum/서버/DI 조립 책임
- 부팅: config → trace → db → container → rbac load → seed admin → http server

## Crate 책임

| Crate | 위치 | 역할 |
|---|---|---|
| `shinespark` | `./shinespark/` | DB 추상화, config, 암호, tracing, HTTP 서버 부팅, error hub, util |
| `shinespark-identity` | `./shinespark-identity/` | User/Auth/RBAC/JWT 도메인. entity + usecase trait + infra impl |
| `shinespark-app` | `./shinespark-app/` | 바이너리. 모든 걸 조립하고 Axum 서버 실행 |

자세한 내용은 각 `crates/*.md` 참조.

## 의존 방향

```
shinespark-app
    └── shinespark-identity
            └── shinespark
```

- 역방향 의존 금지 (core 가 domain 을 모름, domain 이 app 을 모름)
- `shinespark-identity` 은 `axum` / `tower-sessions` 등 HTTP 프레임워크 의존 금지
- `shinespark` 은 도메인 타입 (User 등) 을 모름

## Layering (hexagonal / clean)

내부 → 외부 순서:

1. **Entity** — `shinespark-identity/src/entities.rs`. 값 객체/집합체. framework-free
2. **Usecase trait (port)** — `shinespark-identity/src/usecases/`. async trait. 입출력은 Command/Query DTO
3. **Repository trait (port)** — `shinespark-identity/src/repositories/`. DB 계약만 정의
4. **Infra impl (adapter)** — `shinespark-identity/src/infra/`.
   - `Default*Usecase` = 비즈니스 로직, repository 오케스트레이션
   - `Sqlx*Repository` = SQL 실행 (with `Handle`)
   - `Mock*Repository` = 테스트용 in-memory
5. **App 조립** — `shinespark-app/src/main.rs::AppContainer` 가 `Arc<dyn Trait>` 필드로 보관

규칙: 내부 레이어는 외부 레이어를 모른다. 외부만 내부에 의존한다.

## 부팅 시퀀스 (`shinespark-app/src/main.rs`)

```
1. AppConfig::load_dotenv()        # .env + configs/shinespark-<mode>.env 읽기
2. AppConfig::new()                # configs/shinespark.toml + env overlay
3. shinespark::trace::init(&cfg)   # tracing 초기화 (console/file)
4. shinespark::db::Database::new() # sqlx 커넥션 풀
5. AppContainer::new(db, &cfg)     # Arc<dyn Trait> 모두 조립
6. container.rbac_usecase.load()   # role/permission 캐시 워밍
7. seed_admin(...)                 # 개발용 admin 계정 보장
8. axum::Router + session layer
9. shinespark::http::run(router)   # host:port 바인드 후 서빙
```

각 단계 상세는 `./crates/shinespark-app.md`.

## Config 계층

```
configs/shinespark.toml           # base
configs/shinespark-<RUN_MODE>.env # dev/local/production overlay
.env (루트)                        # 최상위 overlay
APP__<section>__<key>=...         # env var 최종 overlay
```

- `RUN_MODE` env 로 모드 선택 (기본 `local`)
- 릴리스 빌드에서는 실행 파일 기준 경로 탐색
- 자세한 내용은 `docs/02. config.md` 와 `./crates/shinespark-core.md`

## Feature flags (DB driver)

`shinespark/Cargo.toml` 기준:

| Feature | 기본 | DB |
|---|---|---|
| `db-driver-postgres` | ✅ | PostgreSQL |
| `db-driver-sqlite` |  | SQLite |
| `db-driver-mysql` |  | MySQL |

**상호 배타** — 동시에 하나만 활성. `shinespark-identity/sql/` 의 SQL 은 Postgres 기준으로 작성되므로 다른 driver 를 쓸 경우 SQL 호환을 직접 확인해야 한다.

## Error 전파

- `shinespark::Error` 단일 enum 이 모든 crate 를 관통
- `shinespark-identity` 는 이 enum 에 자기 도메인 에러를 매핑 (`InvalidCredentials`, `NotFound` 등)
- `shinespark-app` 의 `ApiError` 가 HTTP status 로 최종 변환 (`./http-layer.md`)

## 테스트 전략

- 순수 단위 테스트: `MockUserRepository` + `Default*Usecase` 조합
- 통합 테스트 (DB 필요): `#[ignore]` 마크 — `cargo test -- --ignored` 로 실행
- env 를 건드리는 테스트: `#[serial]` (crate `serial_test`)

## 도메인 추가 시

현재 도메인은 identity 하나. 새 도메인 (예: `shinespark-billing`) 을 추가할 경우:

1. crate 신설, `shinespark` 에만 의존
2. 동일 layering 규칙 (entity → usecase → repo → infra impl) 따름
3. `shinespark-app` 의 `AppContainer` 에 필드 추가 + 부팅 시퀀스에 조립
4. `_index.md`, 이 파일, 신규 `crates/<name>.md` 생성 — `MAINTENANCE.md` 참고
