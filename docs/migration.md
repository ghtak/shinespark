# Database Migration Guide (SQLX)

이 문서는 `sqlx-cli`를 사용하여 Shinespark 프로젝트의 데이터베이스 스키마를 관리하는 방법을 설명합니다. 모든 마이그레이션 파일은 프로젝트 루트의 `migrations/` 디렉토리에 위치합니다.

## 1. 환경 설정 (Database URL)

`sqlx` 명령어를 실행하기 위해서는 `DATABASE_URL` 환경 변수가 설정되어 있어야 합니다.

### Linux / macOS (Bash, Zsh)
```bash
export DATABASE_URL="postgres://username:password@localhost:5432/shinespark"
```

### Windows (PowerShell)
```powershell
$env:DATABASE_URL = "postgres://username:password@localhost:5432/shinespark"
```

---

## 2. 마이그레이션 도구 사용법

### 마이그레이션 파일 생성

#### 일반 (순방향 전용)
기본적으로 단일 SQL 파일이 생성되며, 역방향(Down) 정보가 없어 `revert`가 불가능합니다.
```bash
sqlx migrate add <description>
```

#### Reversible (순방향 + 역방향)
`-r` 또는 `--reversible` 플래그를 사용하면 `.up.sql`과 `.down.sql` 한 쌍이 생성됩니다. `revert` 명령어를 사용하려면 이 방식으로 생성해야 합니다.
```bash
sqlx migrate add -r <description>
```
명령 실행 시 `YYYYMMDDHHMMSS_description.up.sql`과 `YYYYMMDDHHMMSS_description.down.sql` 파일이 생성됩니다.

### 마이그레이션 적용 (Run)
생성된 SQL 파일을 데이터베이스에 실제로 반영합니다.
```bash
sqlx migrate run
```
이 명령어는 아직 실행되지 않은 모든 마이그레이션을 순차적으로 실행하고, 적용된 정보는 DB의 `_sqlx_migrations` 테이블에 기록됩니다.

### 마이그레이션 롤백 (Revert)
가장 최근에 적용된 마이그레이션 하나를 되돌립니다.
```bash
sqlx migrate revert
```
> [!WARNING]
> `revert`는 해당 마이그레이션의 `.down.sql` 내용을 실행합니다. 생성 시 `-r` 플래그를 쓰지 않았다면 되돌릴 수 없으며, 롤백은 데이터 손실을 초래할 수 있으므로 주의해서 사용해야 합니다.

---

## 3. Shinespark 프로젝트 내 자동 마이그레이션

개발 편의성을 위해 애플리케이션 시작 시점에 마이그레이션을 자동으로 수행하도록 설정할 수 있습니다.

### 코드 예시 (Rust)
`shinespark` 라이브러리 또는 `shinespark_app`의 초기화 시점에 아래 로직을 추가합니다.

```rust
use sqlx::postgres::PgPool;

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    // migrations/ 폴더 내의 SQL 파일을 바이너리에 포함하여 실행합니다.
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
}
```

---

## 4. 유용한 팁

- **컴파일 타임 체크:** `sqlx`는 컴파일 시점에 SQL 구문을 실제 DB와 대조하여 검증합니다. 이를 위해 `.env` 파일에 `DATABASE_URL`을 작성해두면 편리합니다.
- **버전 관리:** 마이그레이션 SQL 파일은 절대 수정하지 마십시오. 변경이 필요하면 항상 새로운 마이그레이션 파일을 추가해야 합니다.
