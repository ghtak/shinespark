use axum::http::Request;
use axum::response::IntoResponse;
use opentelemetry::trace::TraceContextExt;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::Instrument;
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

fn trace_span<B>(req: &Request<B>) -> (String, tracing::Span) {
    let parent_context =
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderMapExtractor(req.headers()))
        });

    let span = tracing::info_span!(
        "http.request",
        method = %req.method(),
        uri = %req.uri(),
        trace_id = tracing::field::Empty,
    );
    // trace_id auto generate if not present (otel)
    span.set_parent(parent_context);
    let trace_id = span.context().span().span_context().trace_id().to_string();
    span.record("trace_id", &trace_id);
    (trace_id, span)
}

fn add_trace_id(response: &mut axum::response::Response, trace_id: String) {
    if let Ok(value) = axum::http::HeaderValue::from_str(&trace_id) {
        response.headers_mut().insert("X-Trace-ID", value);
    }
}

pub async fn trace_layer(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let (trace_id, span) = trace_span(&request);
    let mut response = next.run(request).instrument(span).await;
    add_trace_id(&mut response, trace_id);
    response
}

#[derive(Clone, Default)]
pub struct TraceLayer;

impl<S> Layer<S> for TraceLayer {
    type Service = TraceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceService { inner }
    }
}

#[derive(Clone, Default)]
pub struct TraceService<S> {
    inner: S,
}

impl<B, S> Service<Request<B>> for TraceService<S>
where
    S: Service<Request<B>, Response = axum::response::Response>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: IntoResponse,
    B: Send + 'static,
{
    type Response = axum::response::Response;
    type Error = S::Error;
    type Future = Pin<
        Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let (trace_id, span) = trace_span(&req);
        let mut inner = self.inner.clone();
        Box::pin(
            async move {
                let mut response = inner.call(req).await?;
                add_trace_id(&mut response, trace_id);
                Ok(response)
            }
            .instrument(span),
        )
    }
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

        let app =
            Router::new().route("/", get(|| async { "ok" })).layer(TraceLayer);

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
