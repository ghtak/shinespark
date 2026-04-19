use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Response},
    middleware::Next,
};
use sqlx::types::chrono::Utc;
use tracing::{Instrument, info_span};

const TRACE_ID_HEADER: &str = "x-trace-id";
const SPAN_NAME: &str = "http.request";

tokio::task_local! {
    static CURRENT_TRACE_ID: String;
}

#[allow(dead_code)]
pub fn get_current_trace_id() -> Option<String> {
    CURRENT_TRACE_ID.try_with(|id| id.clone()).ok()
}

pub async fn trace_id_middleware(req: Request, next: Next) -> Response<Body> {
    let trace_id = req
        .headers()
        .get(TRACE_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let now = Utc::now().format("%m%d-%H%M%S%.3f").to_string();
            let random = nanoid::nanoid!(8);
            format!("{}-{}", now, random)
        });
    let span = info_span!(SPAN_NAME, %trace_id);
    let mut response =
        CURRENT_TRACE_ID.scope(trace_id.clone(), next.run(req).instrument(span)).await;
    if let Ok(header_value) = HeaderValue::from_str(&trace_id) {
        response.headers_mut().insert(TRACE_ID_HEADER, header_value);
    }
    response
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, routing::get};
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn test_get_current_trace_id() {
        let trace_id = get_current_trace_id();
        assert!(trace_id.is_none());
    }

    #[tokio::test]
    async fn test_trace_id_middleware_with_trace_id() {
        let app = axum::Router::new()
            .route(
                "/",
                get(|| async {
                    let trace_id = get_current_trace_id();
                    assert!(trace_id.is_some());
                    (StatusCode::OK, "index")
                }),
            )
            .layer(axum::middleware::from_fn(middlewares::trace_id_middleware));

        let response =
            app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get(TRACE_ID_HEADER).is_some());
    }
}
