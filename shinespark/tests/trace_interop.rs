use axum::{Router, routing::get};
use opentelemetry::trace::TraceContextExt;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use shinespark::http::middleware::trace_layer;
use shinespark::trace::get_current_trace_id;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[tokio::test]
async fn test_trace_propagation_interop() {
    shinespark::trace::init(&shinespark::config::TraceConfig::default())
        .expect("Failed to initialize tracing");

    // --- 서버 B (요청을 받는 서버) ---
    // trace_layer 미들웨어가 적용되어 전파된 ID를 추출합니다.
    let app_b = Router::new()
        .route(
            "/target",
            get(|| async {
                let trace_id = get_current_trace_id()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string());

                tracing::info!("trace_id: {}", trace_id);
                // 응답 바디에 Trace ID를 담아 클라이언트가 확인할 수 있게 함
                trace_id
            }),
        )
        .layer(axum::middleware::from_fn(trace_layer));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app_b).await.unwrap();
    });

    // --- 서버 A (요청을 보내는 클라이언트 역할) ---
    // 핵심: Subscriber에 OpenTelemetryLayer를 등록해야 span.context()가 작동합니다.
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let _ = tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer())
        .try_init();

    // TracingMiddleware를 추가하여 자동으로 'traceparent' 헤더를 생성하게 함
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(
            TracingMiddleware::<reqwest_tracing::DefaultSpanBackend>::default(),
        )
        .build();

    // 클라이언트 쪽에서도 Span이 활성화되어 있어야 미들웨어가 ID를 찾아 헤더에 넣습니다.
    let span = tracing::info_span!("client_request");

    // Span 컨텍스트 내에서 요청 실행을 위해 instrument 사용
    use tracing::Instrument;
    let trace_id_in_b = async move {
        client
            .get(format!("http://{}/target", addr))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }
    .instrument(span.clone())
    .await;

    // 클라이언트 Span의 Trace ID 추출
    let span_context = span.context().span().span_context().clone();
    let client_trace_id = span_context.trace_id().to_string();

    assert_ne!(
        trace_id_in_b, "none",
        "Server B should have received a Trace ID"
    );
    assert_eq!(
        trace_id_in_b, client_trace_id,
        "Trace ID should be propagated from Client to Server"
    );
}
