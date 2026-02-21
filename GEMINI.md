# Shinespark Project Context

## Project Overview
Shinespark is a Rust-based application workspace featuring a layered architecture with a focus on Identity Management (RBAC) and high observability through OpenTelemetry.

### Core Components
- **`shinespark`**: The foundational library providing cross-cutting concerns:
  - **Database**: Custom `db::Handle` for Unit of Work (UOW) and transaction management using `sqlx`.
  - **Error Handling**: Centralized `Error` and `Result` types.
  - **Observability**: OpenTelemetry (OTLP) integration and tracing middleware.
  - **HTTP**: Middleware for Axum (response mapping, tracing).
- **`shinespark-identity`**: Domain-specific logic for authentication and authorization.
  - Implements RBAC (Roles, Permissions).
  - Uses Argon2 for secure password hashing.
  - Follows a strict Repository/Service pattern.
- **`shinespark-app`**: The executable entry point.
  - Built with the Axum web framework.
  - Orchestrates the domain services into a functional web API.

## Tech Stack
- **Language**: Rust (Edition 2024)
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Database**: [PostgreSQL](https://www.postgresql.org/) with [SQLx](https://github.com/launchbadge/sqlx)
- **Observability**: [OpenTelemetry](https://opentelemetry.io/) (OTLP), [Tracing](https://github.com/tokio-rs/tracing)
- **Configuration**: [config-rs](https://github.com/mehcode/config-rs) (TOML based)
- **Security**: Argon2 for hashing

## Architecture & Conventions

### Layered Architecture (UOW Pattern)
1.  **UseCase Layer**: Orchestrates business logic and manages transaction boundaries using `db::Handle`.
2.  **Service Layer**:
    - `Service`: Stateless/Read-only operations using `sqlx::Pool`.
    - `ServiceTx`: Atomic/Transactional operations requiring `db::Handle`.
3.  **Repository Layer**: Low-level CRUD operations. All methods MUST accept `db::Handle` to participate in transactions.

### Development Principles
- **Plan-First, Code-Later**: Always propose a markdown checklist plan before implementing code.
- **Contract-First**: Define `Interface` (Trait), `Type Definition`, or `Abstract Class` before writing logic.
- **Atomic Tasks**: Break down work into small, verifiable units. One logical unit per PR/iteration.
- **Traceability**: All major operations should be wrapped in tracing spans.

## Building and Running

### Prerequisites
- Rust (latest stable)
- Docker Compose (for PostgreSQL and OpenTelemetry Collector)

### Commands
- **Setup Infrastructure**:
  ```powershell
  docker-compose -f deploy/docker-compose.yml up -d
  ```
- **Database Migrations**:
  ```powershell
  # Requires sqlx-cli
  sqlx migrate run
  ```
- **Build**:
  ```powershell
  cargo build
  ```
- **Run Application**:
  ```powershell
  cargo run -p shinespark-app
  ```
- **Test**:
  ```powershell
  cargo test
  ```
  *Note: Some database tests may require `DATABASE_URL` to be set.*

- **Linting**:
  ```powershell
  cargo fmt
  cargo clippy
  ```

## Project Structure
```text
C:\Users\tlab\works\shinespark\
├── configs/             # Configuration files (default, dev, test)
├── deploy/              # Deployment manifests (Docker Compose)
├── docs/                # Architectural and domain documentation
├── migrations/          # SQLx database migrations
├── shinespark/          # Core library (Shared utilities)
├── shinespark-identity/ # Identity & RBAC domain crate
└── shinespark-app/      # Main application entry point
```

---

## AI Agent Mandates (Strict)

### 1. 작업 원칙: [Plan-First, Code-Later]
- **무조건 플랜 선행**: 코드를 작성하기 전, 반드시 작업 계획(Plan)을 마크다운 체크리스트 형태로 제안하고 사용자의 리뷰를 받으십시오.
- **구조적 접근**: 폴더 구조와 파일 레이아웃을 먼저 확정한 후 개별 파일 작업을 진행합니다.
- **마이크로 단위 진행**: 한 번의 응답에 너무 많은 파일을 생성하지 마십시오. 하나의 논리적 단위(Atomic Task)가 완료될 때마다 컨펌을 받습니다.

### 2. 개발 단계별 가이드라인
- **Phase 1: 구조 구성 (Architecture)**: 전체적인 디렉토리 구조와 의존성 그래프를 설계합니다.
- **Phase 2: 인터페이스 정의 (Contract First)**: 실제 로직을 짜기 전, Interface, Trait, Type Definition을 먼저 작성합니다.
- **Phase 3: 실행 가능한 목업 (Mock-up & TDD)**: 인터페이스를 바탕으로 Dummy Data나 가짜 로직을 포함한 '우선 동작하는 코드'를 작성합니다.
- **Phase 4: 실제 구현 (Implementation)**: 목업 로직을 실제 비즈니스 로직, DB 접근 코드로 교체합니다.
