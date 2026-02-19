use std::sync::OnceLock;

use crate::config::{TraceConfig, TraceFormat};
use opentelemetry::trace::TracerProvider;
use opentelemetry::trace::{TraceContextExt, TraceId};
use opentelemetry::{Context, trace::TraceResult};
use opentelemetry_sdk::export::trace::SpanData;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{Span, SpanProcessor};
use rand::Rng;
use rand::rng;
use tracing::Subscriber;
use tracing_appender::{non_blocking::WorkerGuard, rolling::daily};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;

static LOGGING_INIT: OnceLock<Vec<WorkerGuard>> = OnceLock::new();

fn new_fmt_layer<S>(
    filter: EnvFilter,
    format: TraceFormat,
    writer: tracing_appender::non_blocking::NonBlocking,
) -> Box<dyn tracing_subscriber::Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    match format {
        TraceFormat::Full => {
            fmt::layer().with_writer(writer).with_filter(filter).boxed()
        }
        TraceFormat::Compact => fmt::layer()
            .with_writer(writer)
            .compact()
            .with_filter(filter)
            .boxed(),
        TraceFormat::Pretty => fmt::layer()
            .with_writer(writer)
            .pretty()
            .with_filter(filter)
            .boxed(),
        TraceFormat::Json => {
            fmt::layer().with_writer(writer).json().with_filter(filter).boxed()
        }
    }
}

pub fn init(logging_config: &TraceConfig) -> crate::Result<()> {
    let mut setup_result = Ok(());
    LOGGING_INIT.get_or_init(|| {
        let mut guards = Vec::new();
        let mut layers = Vec::new();

        // 1. Console Layer
        if let Some(console_config) = logging_config.console.as_ref() {
            let (console, console_guard) =
                tracing_appender::non_blocking::NonBlockingBuilder::default()
                    .buffered_lines_limit(console_config.buffer_limit)
                    .lossy(console_config.lossy)
                    .finish(std::io::stdout());
            guards.push(console_guard);

            let filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&console_config.filter));

            layers.push(new_fmt_layer(filter, console_config.format, console));
        }

        // 2. File Layer
        if let Some(file_config) = logging_config.file.as_ref() {
            let (file_writer, file_guard) =
                tracing_appender::non_blocking::NonBlockingBuilder::default()
                    .buffered_lines_limit(file_config.buffer_limit)
                    .lossy(file_config.lossy)
                    .finish(daily(
                        file_config.directory.as_str(),
                        file_config.filename.as_str(),
                    ));
            guards.push(file_guard);

            let filter = EnvFilter::new(&file_config.filter);

            layers.push(new_fmt_layer(filter, file_config.format, file_writer));
        }
        // 3. OTel Layer

        if let Some(_otel_config) = logging_config.otel.as_ref() {
            // let tracer = opentelemetry_otlp::new_pipeline()
            //     .tracing()
            //     .with_exporter(
            //         opentelemetry_otlp::new_exporter()
            //             .tonic()
            //             .with_endpoint(&otel_config.endpoint),
            //     )
            //     .install_batch(opentelemetry_sdk::runtime::Tokio)
            //     .expect("failed to install otop pipeline");

            // let filter = EnvFilter::new(&otel_config.filter);

            // layers.push(
            //     tracing_opentelemetry::layer()
            //         .with_tracer(tracer)
            //         .with_filter(filter)
            //         .boxed(),
            // );
        } else {
            // trace id 통합용도
            let otel_provider =
                opentelemetry_sdk::trace::TracerProvider::builder()
                    .with_span_processor(NoopProcessor)
                    .build();
            opentelemetry::global::set_tracer_provider(otel_provider.clone());
            let telemetry_layer = tracing_opentelemetry::layer()
                .with_tracer(otel_provider.tracer("shinespark"));
            layers.push(telemetry_layer.boxed());
        }

        opentelemetry::global::set_text_map_propagator(
            TraceContextPropagator::new(),
        );

        if let Err(e) = tracing_subscriber::registry().with(layers).try_init() {
            setup_result = Err(anyhow::Error::new(e)
                .context("failed to init tracing")
                .into());
        }
        guards
    });
    setup_result
}

#[derive(Debug)]
pub struct NoopProcessor;

impl SpanProcessor for NoopProcessor {
    fn on_start(&self, _span: &mut Span, _cx: &Context) {}
    fn on_end(&self, _span: SpanData) {}
    fn force_flush(&self) -> TraceResult<()> {
        Ok(())
    }
    fn shutdown(&mut self) -> TraceResult<()> {
        Ok(())
    }
}

pub fn get_current_trace_id() -> Option<TraceId> {
    let span = tracing::Span::current();
    let context = span.context();
    let span_context = context.span().span_context().clone();

    if span_context.is_valid() {
        Some(span_context.trace_id())
    } else {
        None
    }
}

pub fn generate_trace_id() -> TraceId {
    let mut rng = rng();
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    TraceId::from_bytes(bytes)
}

pub fn get_trace_id_string() -> String {
    get_current_trace_id()
        .map(|id| id.to_string())
        .unwrap_or_else(|| generate_trace_id().to_string())
}

#[cfg(test)]
mod tests {
    use crate::config::TraceConsoleConfig;

    use super::*;

    async fn inner_span() {
        tracing::info!("inside inner span");
        tracing::error!("inside inner span");
    }

    fn default_config() -> TraceConfig {
        TraceConfig {
            console: Some(TraceConsoleConfig {
                filter: "debug".into(),
                format: crate::config::TraceFormat::Compact,
                buffer_limit: 256_000,
                lossy: true,
            }),
            file: None,
            otel: None,
        }
    }

    #[tokio::test]
    async fn test_logging_basic() {
        let _ = init(&default_config());
        tracing::debug!("debug message");
        tracing::info!("info message");
        tracing::warn!("warn message");
        tracing::error!("error message");

        let span = tracing::info_span!("my_span", foo = 3);
        let _enter = span.enter();
        inner_span().await;
        tracing::info!("after inner span");
    }

    #[tokio::test]
    async fn test_nested_span() {
        let _ = init(&default_config());

        let span = tracing::info_span!("root_span", foo = 3);
        let _enter = span.enter();
        tracing::debug!("root_span line");

        let child_span = tracing::info_span!("inner_span", foo = 4);
        let _enter2 = child_span.enter();
        tracing::debug!("inner_span line");
        drop(_enter2);
        tracing::debug!("back in root_span");
    }

    #[tokio::test]
    async fn test_file_logging_config() {
        let temp_dir = "target/test_logs";
        let config = TraceConfig {
            console: Some(TraceConsoleConfig {
                filter: "debug".into(),
                format: crate::config::TraceFormat::Compact,
                buffer_limit: 256_000,
                lossy: true,
            }),
            file: Some(crate::config::TraceFileConfig {
                filter: "debug".into(),
                directory: temp_dir.into(),
                filename: "test_malt.log".into(),
                format: crate::config::TraceFormat::Json,
                buffer_limit: 256_000,
                lossy: true,
            }),
            otel: None,
        };

        // Note: Only the first call to init() actually sets the global logger.
        // Subsequent calls are ignored due to OnceLock.
        let _ = init(&config);

        tracing::info!("test file logging event");

        // We can at least verify the directory exists if this was the first init call
        if std::path::Path::new(temp_dir).exists() {
            println!("Log directory {} exists", temp_dir);
        }
    }
}
