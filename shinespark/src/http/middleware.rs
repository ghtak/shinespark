use axum::{extract::Request, middleware::Next, response::Response};
use opentelemetry::trace::TraceContextExt;
use tracing_opentelemetry::OpenTelemetrySpanExt;

struct HeaderMapExtractor<'a>(&'a axum::http::HeaderMap);

impl<'a> opentelemetry::propagation::Extractor for HeaderMapExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

pub async fn trace_layer(request: Request, next: Next) -> Response {
    let parent_context =
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderMapExtractor(request.headers()))
        });

    let span = tracing::info_span!(
        "http_request",
        method = %request.method(),
        uri = %request.uri(),
        trace_id = tracing::field::Empty,
    );
    // trace_id auto generate if not present (otel)
    span.set_parent(parent_context);

    let trace_id = span.context().span().span_context().trace_id().to_string();
    span.record("trace_id", &trace_id);

    let _enter = span.enter();
    let response = next.run(request).await;

    let mut response = response;
    if let Ok(value) = axum::http::HeaderValue::from_str(&trace_id) {
        response.headers_mut().insert("X-Trace-ID", value);
    }

    response
}

#[cfg(test)]
mod tests {
    use crate::config::TraceConfig;

    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    fn init_test_tracing() {
        crate::trace::init(&TraceConfig::default()).unwrap();
    }

    #[tokio::test]
    async fn test_trace_layer_generates_id() {
        init_test_tracing();

        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(trace_layer));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let header = response.headers().get("X-Trace-ID").unwrap();
        assert!(!header.is_empty());
        // W3C TraceID is 32 chars hex
        assert_eq!(header.to_str().unwrap().len(), 32);
    }

    #[tokio::test]
    async fn test_trace_layer_propagates_id() {
        init_test_tracing();

        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(trace_layer));

        // Valid W3C traceparent: 00-{trace_id}-{parent_id}-{flags}
        let trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
        let traceparent = format!("00-{}-00f067aa0ba902b7-01", trace_id);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("traceparent", traceparent)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let header = response.headers().get("X-Trace-ID").unwrap();
        assert_eq!(header.to_str().unwrap(), trace_id);
    }
}
