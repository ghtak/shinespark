---
scope: 모든 map 파일이 공유하는 헤더 스키마와 작성 규칙
when-to-read: 새 map 파일을 만들거나 기존 파일을 수정할 때
budget: 100
related:
  - ./_index.md
  - ./MAINTENANCE.md
updated: 2026-04-18
---

## TL;DR

- 본문은 한국어, 식별자/경로/헤더/코드는 영문 유지 (grep 친화)
- 파일 첫 블록은 반드시 front-matter + `## TL;DR` (최대 10 불릿)
- 심볼은 본문 복붙 금지 — `file.rs:Symbol` 링크로만 표시
- `budget`(최대 라인) 을 넘기면 분할 또는 하위 map 분리
- 중복 금지: 전역 불변식은 `_index.md` 에만 기술하고 다른 파일은 참조

## Front-matter 스키마

모든 map 파일은 아래 YAML-like 블록으로 시작한다:

```
---
scope: <파일이 다루는 범위, 한 줄>
when-to-read: <AI 가 이 파일을 열어야 하는 상황/트리거>
budget: <최대 라인 수, 숫자>
related:
  - <상대경로>
  - <상대경로>
updated: YYYY-MM-DD
---
```

필드 의미:

| 필드 | 의미 |
|---|---|
| `scope` | 파일이 커버하는 영역. 한 줄. 중복 읽기를 줄이는 필터 |
| `when-to-read` | "지금 이 일을 하는 중이라면 열어라" 형태의 트리거 문장 |
| `budget` | 파일 최대 라인 수 (front-matter 포함). 토큰 예산 상한 |
| `related` | 같이 읽으면 유용한 다른 map 파일의 상대경로 |
| `updated` | 마지막 수정 날짜 (YYYY-MM-DD). 드리프트 감지 기준 |

## 문서 구조 순서

front-matter 바로 다음에 `## TL;DR` (최대 10 불릿). 트리거에 해당하지 않으면 AI 는 여기서 멈춘다. 이후 상세 섹션은 자유 순서이나 다음을 권장:

1. `## TL;DR`
2. `## 범위 / 비범위` (선택)
3. 본문 섹션들 — 중요도/흐름 순
4. `## 관련 링크` (선택; front-matter `related` 와 중복되면 생략)

## 링크 스타일

- 소스 심볼: `shinespark-identity/src/usecases/user_usecase.rs:UserUsecase`
- 라인 번호 고정 금지 (리팩터링에 취약). 심볼명으로 고정.
- map 간 링크: `../domain/identity-usecases.md` 같은 상대 경로
- 외부 문서/이슈는 `docs/*.md` 를 가리키거나 루트 `CLAUDE.md` 를 참조

## 분량 원칙

- 코드 본문 복사 금지. 시그니처는 1-2줄 요약 + 링크
- 전수 나열(모든 SQL 파일, 모든 필드) 대신 네이밍 규칙 + 대표 예시
- 튜토리얼성 설명 금지 (그런 내용은 `docs/plan/` 또는 CLAUDE.md 소관)
- 예산 초과 시 하위 map 으로 분할하고 상위에서 링크

## 언어 규칙

- 설명문(산문): 한국어
- 식별자, 타입명, 경로, 파일명, 코드 블록: 영문 그대로
- 헤더 `## TL;DR`, `## 범위` 등은 한국어 가능, 코드/섹션 앵커는 영문 권장
- 예: "`UserUsecase` 는 사용자 생성/조회/수정 유스케이스를 정의한다."
