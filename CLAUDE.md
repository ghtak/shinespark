# CLAUDE.md

## 0. Context First
- 요청 즉시 코딩 금지. 질문으로 요구사항/제약/스택을 먼저 확정.
- 개념 설계 제안 후 "진행합시다" / "플랜 짜주세요" 승인 전까지 다음 단계 금지.

## 1. Plan-First, Code-Later
- 코딩 전 작업 계획을 `- [ ]` 체크리스트로 제시.
- 폴더 트리·파일 레이아웃 먼저 확정.
- Atomic Task: 한 응답에 하나의 논리 단위. 완료마다 컨펌.

## 2. Development Phases
- Phase 1 Architecture — 디렉토리·의존성 설계
- Phase 2 Contract — Interface / Type / Trait 선언 먼저
- Phase 3 Mock-up — 인터페이스 기반 최소 코드 (+ 필요 시 Unit Test)
- Phase 4 Implementation — Mock → 실 DB / API 교체

## 3. Communication
- 간결성: 핵심 요약 위주, 장황한 설명 지양.
- 변경 요약: 수정·생성 후 3줄 이내 보고.
- 리스크 고지: 설계 결함·병목 발견 시 즉시 제언.
- 상태 추적: 체크리스트 진행을 매 응답 업데이트.

## Project Map

프로젝트 구조·크레이트·레이어·부팅·HTTP·SQL 규약은 **`docs/maps/_index.md`** 에서 진입한다. 세션 시작 시 이 파일부터 읽고, 각 하위 map 의 `when-to-read` 트리거에 해당할 때만 추가로 열어 컨텍스트를 아낀다. map 갱신은 사용자 요청 시에만 (`docs/maps/MAINTENANCE.md`).

## Build & Test

```bash
cargo build
cargo test -- --nocapture
cargo test --package shinespark --lib -- config::tests::test_load_env --exact --nocapture
cargo run --package shinespark-app
cargo check
cargo fmt
```

- env 를 건드리는 테스트는 `#[serial]` (crate `serial_test`)
- 실 DB 통합 테스트는 `#[ignore]` + `cargo test -- --ignored` (`DATABASE_URL` 필요)
