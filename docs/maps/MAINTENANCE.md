---
scope: map 파일 갱신 정책과 드리프트 점검 절차
when-to-read: 사용자가 "map 업데이트" 또는 이에 준하는 요청을 했을 때
budget: 120
related:
  - ./_index.md
  - ./CONVENTIONS.md
updated: 2026-04-18
---

## TL;DR

- **자동 갱신 금지** — 사용자가 명시적으로 요청한 경우에만 수정
- 코드 변경 PR 에 map 갱신을 섞지 않음 (별도 요청으로 처리)
- 갱신 요청 시 트리거 매트릭스를 보고 수정 범위를 결정
- 수정한 파일은 front-matter 의 `updated` 를 오늘 날짜로 갱신
- 함수 본문 복붙·CLAUDE.md 규칙 복제 금지

## 갱신 정책

이 프로젝트의 map 은 **사용자 요청 시에만** 갱신한다. 코드 변경을 하면서 AI 가 자동으로 map 을 수정하지 않는다. 이유는 map 이 AI 세션의 "안정적 참조점" 이어야 하고, 무분별한 자동 수정은 신뢰도를 떨어뜨리기 때문이다.

사용자가 다음과 같이 요청하면 갱신한다:

- "map 업데이트해줘"
- "docs/maps 를 현재 코드와 동기화해줘"
- "<특정 map 파일> 갱신해줘"

요청이 없을 때 AI 가 map 과 코드의 불일치를 발견하면, 작업을 중단하지 말고 사용자에게 한 줄로 알려 후속 판단을 넘긴다: "map 과 코드가 어긋난 부분을 발견했습니다 — 갱신을 원하시면 알려주세요."

## 트리거 매트릭스

아래 코드 변경이 발생한 뒤 map 갱신 요청을 받으면 다음 파일을 본다:

| 코드 변경 | 갱신 대상 map |
|---|---|
| 새 crate 추가 | `_index.md`, `architecture.md`, `crates/<new>.md` 신규 |
| crate 간 의존 방향 변경 | `architecture.md` |
| 새 usecase trait | `domain/identity-usecases.md`, `crates/shinespark-identity.md` |
| 새 repository trait | `domain/identity-repositories.md`, `crates/shinespark-identity.md` |
| 새 `Sqlx*` / `Mock*` impl | `domain/identity-repositories.md` |
| 새 `Default*Usecase` impl | `domain/identity-usecases.md` |
| 새 SQL 디렉터리/파일 규약 변경 | `domain/identity-sql.md` |
| 새 테이블/스키마 | `domain/identity-sql.md`, `_index.md` (불변식) |
| 새 HTTP 라우트/extractor | `http-layer.md`, `crates/shinespark-app.md` |
| `ApiResponse`/`ApiError` 포맷 변경 | `http-layer.md` |
| `AppContainer` 필드 변경 | `crates/shinespark-app.md` |
| 부팅 시퀀스 변경 (`main.rs`) | `crates/shinespark-app.md` |
| config/trace/db 핵심 API 변경 | `crates/shinespark-core.md` |
| feature flag 추가/제거 | `crates/shinespark-core.md`, `_index.md` |
| feature 추가 전체 흐름 변경 | `feature-lifecycle.md` |

여러 파일이 걸리면 각 파일의 `updated` 를 모두 갱신한다.

## 드리프트 점검 절차

사용자가 "전체 동기화" 를 요청하면:

1. **날짜 기준 점검**: `updated` 가 60 일 이상 지난 파일 목록 확보
   ```
   rg "^updated:" docs/maps -n
   ```
2. **심볼 유효성 점검**: 각 map 이 참조한 `file.rs:Symbol` 이 현재 코드에 존재하는지 샘플링
   - trait 이름 (`UserUsecase`, `RbacUsecase` 등)
   - 구조체 이름 (`AppContainer`, `Handle` 등)
   - 테이블명 (`shs_iam_*`)
3. **예산 초과 점검**: 각 파일 라인 수가 `budget` 이하인지
   ```
   find docs/maps -name '*.md' -exec wc -l {} +
   ```
4. **중복 점검**: 전역 불변식이 여러 파일에 중복 서술되지 않는지 — `_index.md` 에만 있어야 함

## 금지 사항

- **함수/구조체 본문 복붙** — 심볼 링크만
- **CLAUDE.md 규칙 재서술** — CLAUDE.md 는 규칙, map 은 내비게이션
- **현재 세션의 임시 TODO/결정** — 이런 내용은 `docs/plan/` 소관
- **grep 으로 찾을 수 있는 전수 나열** — 규칙 + 대표 예시로 대체

## 새 map 파일 추가 절차

1. `CONVENTIONS.md` front-matter 스키마 준수
2. `_index.md` "Where to go" 테이블에 한 줄 추가
3. 관련 파일들의 `related` 필드에 역링크 추가
4. 직접 연결된 `architecture.md` 또는 `feature-lifecycle.md` 에 필요 시 언급
