---
scope: 관리자 페이지 UI 레이어 — SSR 템플릿 구조, CDN 스택, React 아일랜드 패턴
when-to-read: admin 페이지 추가·수정, 템플릿 레이아웃 변경, CDN 버전 업데이트
budget: 280
related:
  - ./_index.md
  - ./http-layer.md
  - ./crates/shinespark-app.md
updated: 2026-04-26
---

## TL;DR

- MiniJinja SSR + CDN React 아일랜드 패턴. 빌드 툴 없음.
- Tailwind CSS (CDN) + Flowbite (CDN) 로 UI 구성.
- 모든 admin 페이지는 `templates/admin/_base.html` 을 상속.
- 인터랙티브 컴포넌트만 React 마운트 포인트(`<div id="...">`)로 분리.
- Admin 라우트는 `/admin/*`, 핸들러는 `CurrentUser` 추출기로 인증.

---

## CDN 버전 (고정)

| 라이브러리 | 버전 | 용도 |
|---|---|---|
| Tailwind CSS | 3.x (play CDN) | 유틸리티 CSS |
| Flowbite | 2.3.0 | Tailwind 기반 컴포넌트 |
| React | 18.x | 인터랙티브 아일랜드 |
| ReactDOM | 18.x | DOM 마운트 |
| Babel Standalone | 7.x | JSX 브라우저 변환 (dev 전용) |

> **주의**: Babel Standalone 은 dev 에서만 허용. prod 에서는 JSX 없이 `React.createElement` 사용.

### CDN URL 목록

```
Tailwind:    https://cdn.tailwindcss.com
Flowbite CSS: https://cdnjs.cloudflare.com/ajax/libs/flowbite/2.3.0/flowbite.min.css
Flowbite JS:  https://cdnjs.cloudflare.com/ajax/libs/flowbite/2.3.0/flowbite.min.js
React (dev):  https://unpkg.com/react@18/umd/react.development.js
ReactDOM(dev):https://unpkg.com/react-dom@18/umd/react-dom.development.js
React (prod): https://unpkg.com/react@18/umd/react.production.min.js
ReactDOM(prod):https://unpkg.com/react-dom@18/umd/react-dom.production.min.js
Babel:        https://unpkg.com/@babel/standalone/babel.min.js
```

---

## 템플릿 파일 레이아웃

```
templates/
  base.html                  ← 전역 base (비 admin 용)
  admin/
    _base.html               ← admin 공통 레이아웃 (sidebar + topbar)
    partials/
      _sidebar.html          ← 사이드바 네비게이션
      _topbar.html           ← 상단 바 (유저 드롭다운 등)
    pages/
      dashboard.html         ← 대시보드
      users.html             ← 사용자 관리
      (추가 페이지...)
```

### 상속 규칙

```jinja2
{# 모든 admin 페이지 #}
{% extends "admin/_base.html" %}

{% block title %}페이지명 | Admin{% endblock %}

{% block content %}
  {# 페이지 콘텐츠 #}
{% endblock %}

{% block scripts %}
  {# 페이지별 JS (React 아일랜드 등) #}
{% endblock %}
```

### `admin/_base.html` 블록 인터페이스

| 블록명 | 위치 | 용도 |
|---|---|---|
| `title` | `<title>` | 페이지 제목 |
| `head_extra` | `</head>` 직전 | 페이지별 추가 CSS |
| `content` | `<main>` 내부 | 페이지 본문 |
| `scripts` | `</body>` 직전 | 페이지별 JS / React 마운트 |

---

## CDN 로딩 순서 (`admin/_base.html`)

```html
<head>
  <!-- 1. Tailwind play CDN -->
  <script src="https://cdn.tailwindcss.com"></script>
  <!-- 2. Flowbite CSS -->
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/.../flowbite.min.css">
  <!-- 3. 페이지별 추가 head -->
  {% block head_extra %}{% endblock %}
</head>
<body>
  ...레이아웃...
  <!-- 4. React + ReactDOM (RUN_MODE 따라 dev/prod 전환) -->
  {% if run_mode == "dev" %}
  <script src="https://unpkg.com/react@18/umd/react.development.js"></script>
  <script src="https://unpkg.com/react-dom@18/umd/react-dom.development.js"></script>
  <script src="https://unpkg.com/@babel/standalone/babel.min.js"></script>
  {% else %}
  <script src="https://unpkg.com/react@18/umd/react.production.min.js"></script>
  <script src="https://unpkg.com/react-dom@18/umd/react-dom.production.min.js"></script>
  {% endif %}
  <!-- 5. Flowbite JS (body 맨 끝) -->
  <script src="https://cdnjs.cloudflare.com/.../flowbite.min.js"></script>
  <!-- 6. 페이지별 스크립트 -->
  {% block scripts %}{% endblock %}
</body>
```

> `run_mode` 는 핸들러에서 `context! { run_mode => ... }` 로 주입하거나 `_base.html` 레벨 전역 컨텍스트로 제공.

---

## React 아일랜드 패턴

### 마운트 포인트 규칙

- SSR HTML 안에 `<div id="island-<name>">` 를 배치.
- `{% block scripts %}` 안에서 `ReactDOM.createRoot` 로 마운트.
- 아일랜드 간 공유 상태 없음 — 각 아일랜드는 독립적.
- 초기 데이터는 `data-*` 속성이나 SSR 인라인 JSON 으로 전달.

### dev 예시 (Babel JSX 허용)

```html
{% block scripts %}
<div id="island-user-table" data-users="{{ users_json }}"></div>

<script type="text/babel">
  const el = document.getElementById('island-user-table');
  const users = JSON.parse(el.dataset.users);

  function UserTable({ users }) {
    return (
      <table className="...">
        {users.map(u => <tr key={u.id}><td>{u.email}</td></tr>)}
      </table>
    );
  }

  ReactDOM.createRoot(el).render(<UserTable users={users} />);
</script>
{% endblock %}
```

### prod 예시 (createElement, JSX 없음)

```html
{% block scripts %}
<div id="island-user-table" data-users="{{ users_json }}"></div>

<script>
  const el = document.getElementById('island-user-table');
  const users = JSON.parse(el.dataset.users);

  function UserTable(props) {
    return React.createElement('table', { className: '...' },
      props.users.map(u =>
        React.createElement('tr', { key: u.id },
          React.createElement('td', null, u.email)
        )
      )
    );
  }

  ReactDOM.createRoot(el).render(React.createElement(UserTable, { users }));
</script>
{% endblock %}
```

---

## Flowbite 컴포넌트 사용 규칙

- Tailwind 유틸리티와 혼용 허용.
- Flowbite JS 초기화는 CDN 로드 후 자동 실행 (data-attribute 방식) — 별도 `initFlowbite()` 호출 불필요.
- React 아일랜드 내부에서 Flowbite 컴포넌트 필요 시 `window.Flowbite` 초기화 충돌 주의 → 해당 컴포넌트는 Tailwind 유틸리티로 직접 구현.

---

## 라우트 규칙

```
/admin/            → 대시보드
/admin/users       → 사용자 관리
/admin/<resource>  → 리소스별 페이지
```

- 핸들러 파일: `shinespark-app/src/http/routes.rs` 내 `pub mod admin` 서브모듈.
- 반환 타입: `Result<TemplateResponse, ApiError>`.
- 인증: `CurrentUser` 추출기 필수 (미인증 시 자동 401).
- 권한: `container.rbac_usecase.check_perm(...)` 핸들러 내 명시 호출.

### 핸들러 기본형

```rust
async fn dashboard(
    State(container): State<Arc<AppContainer>>,
    user: CurrentUser,
) -> Result<TemplateResponse, ApiError> {
    let html = container
        .template_env
        .render("admin/pages/dashboard.html", context! {
            user => user.0,
            run_mode => std::env::var("RUN_MODE").unwrap_or_default(),
        })
        .map_err(|e| shinespark::Error::Internal(anyhow::anyhow!(e)))?;
    Ok(TemplateResponse(html))
}
```

---

## 제약 요약

| 항목 | 규칙 |
|---|---|
| 빌드 툴 | 없음. 번들러·npm 금지 |
| JSX | dev 환경 Babel CDN 한정. prod 금지 |
| 상태 관리 | 아일랜드 범위 내 `useState` 만. 전역 store 금지 |
| CSS | Tailwind + Flowbite 클래스만. 인라인 style 금지 |
| 컴포넌트 파일 분리 | CDN 환경이므로 모듈 분리 없음 — 페이지 `{% block scripts %}` 에 인라인 |
| 초기 데이터 전달 | SSR `data-*` 속성 또는 인라인 JSON (`<script type="application/json">`) |
| 인증 | 모든 admin 핸들러에 `CurrentUser` 필수 |
