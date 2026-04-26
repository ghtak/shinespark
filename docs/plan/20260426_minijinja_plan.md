# MiniJinja SSR 연동 계획

## 목표

`shinespark-app` 에 MiniJinja 템플릿 엔진을 연동해 서버사이드 렌더링(SSR) 지원을 추가한다.

## 제약

- 기존 JSON API 라우트(`/identity/**`) 영향 없음
- 템플릿 파일은 런타임 로드 — 재시작 없이 수정 가능 (dev 편의)
- `AppContainer` 에 `template_env` 를 주입, handler 에서 `State` 로 접근

---

## 파일 레이아웃

```
shinespark-app/
  templates/                        # 신규 — 템플릿 루트
    base.html                       # 공통 레이아웃 (block 상속)
    index.html                      # 예시 페이지
  src/
    http/
      template.rs                   # 신규 — TemplateResponse, init_template_env()
      routes.rs                     # 기존 — SSR 라우트 추가
    main.rs                         # AppContainer 에 template_env 필드 추가
```

---

## 작업 체크리스트

### Phase 1 — 의존성

- [ ] `Cargo.toml` (`shinespark-app`) 에 추가
  ```toml
  minijinja = { version = "2", features = ["loader"] }
  ```

### Phase 2 — 템플릿 환경 초기화

- [ ] `src/http/template.rs` 생성
  - `init_template_env(template_dir: &str) -> Environment<'static>` — 파일시스템 로더 설정
  - `TemplateResponse(String)` 타입 + `IntoResponse` 구현 (`text/html; charset=utf-8`)

### Phase 3 — AppContainer 연동

- [ ] `AppConfig` 에 `[template]` 섹션 추가 — `dir: String` (필수)
  ```toml
  # configs/shinespark.toml
  [template]
  dir = "templates/"
  ```
- [ ] `main.rs:AppContainer` 에 `template_env: Arc<Environment<'static>>` 필드 추가
- [ ] `AppContainer::new(&cfg)` 에서 `init_template_env(&cfg.template.dir)` 호출

### Phase 4 — 라우트 연결

- [ ] `http/routes.rs` 에 SSR 서브모듈 (`web`) 추가
  - `GET /` — `index.html` 렌더링 예시 핸들러
- [ ] `main.rs` 라우터에 `web::routes()` 머지

### Phase 5 — 템플릿 파일

- [ ] `templates/base.html` — `{% block content %}{% endblock %}` 구조
- [ ] `templates/index.html` — `{% extends "base.html" %}` 상속 예시

---

## 핵심 설계

### TemplateResponse

```rust
// http/template.rs
pub struct TemplateResponse(pub String);

impl IntoResponse for TemplateResponse {
    fn into_response(self) -> Response {
        (
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            self.0,
        ).into_response()
    }
}
```

### 핸들러 패턴

```rust
async fn index(State(container): State<Arc<AppContainer>>) -> Result<TemplateResponse, ApiError> {
    let tmpl = container.template_env.get_template("index.html")?;
    let html = tmpl.render(context! { title => "Home" })?;
    Ok(TemplateResponse(html))
}
```

### 에러 처리

`minijinja::Error` → `shinespark::Error::Internal` → `ApiError(500)` 변환 필요  
(`From<minijinja::Error> for shinespark::Error` 추가 또는 `.map_err(...)` 인라인)

---

## 리스크

| 항목 | 내용 |
|---|---|
| 템플릿 경로 | `cfg.template.dir` 로 주입 — 절대/상대 모두 허용, 환경별 `.env` 에서 오버라이드 |
| 에러 타입 누락 | `minijinja::Error` 변환 빠뜨리면 handler 에서 컴파일 에러 |
| 핫리로드 | `RUN_MODE=dev` 일 때만 `path_loader` 로 매 요청 재로드, prod 는 부팅 시 캐시 고정 |
