[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["registry", "env-filter"] }
tracing-opentelemetry = "0.22"
opentelemetry = { version = "0.21", features = ["rt-tokio"] }
opentelemetry_sdk = { version = "0.21", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.14", features = ["grpc-tonic"] } # gRPC 사용 시
tokio = { version = "1", features = ["full"] }


use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. OTLP Exporter 설정 (Collector 주소 지정)
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317"); // Collector의 gRPC 포트

    // 2. Tracer 구성
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
            opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new("service.name", "my-rust-service")]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // 3. Tracing Subscriber에 OTel 레이어 등록
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    // 이제부터 발생하는 span은 Collector로 전송됩니다.
    let _span = tracing::info_span!("main_operation").entered();
    tracing::info!("Hello from Rust with OpenTelemetry!");

    // 종료 전 남은 데이터를 보낼 수 있게 명시적으로 종료 처리
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}


receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317  # Rust 앱이 데이터를 쏘는 곳
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch: # 성능을 위해 데이터를 모아서 전송

exporters:
  logging: # 수집된 데이터를 Collector의 콘솔에 출력 (디버깅용)
    verbosity: detailed
  otlp: # 최종 분석 도구(예: Jaeger, Tempo)로 전송
    endpoint: "jaeger-collector:4317"
    tls:
      insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging, otlp]