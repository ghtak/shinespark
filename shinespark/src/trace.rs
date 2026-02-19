use std::sync::OnceLock;
use std::vec;

use crate::config::{LoggingConfig, LoggingFormat};
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing::Subscriber;
use tracing_appender::{non_blocking::WorkerGuard, rolling::daily};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;

static LOGGING_INIT: OnceLock<Vec<WorkerGuard>> = OnceLock::new();

fn new_fmt_layer<S>(
    format: LoggingFormat,
    writer: tracing_appender::non_blocking::NonBlocking,
) -> Box<dyn tracing_subscriber::Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    match format {
        LoggingFormat::Full => fmt::layer().with_writer(writer).boxed(),
        LoggingFormat::Compact => {
            fmt::layer().with_writer(writer).compact().boxed()
        }
        LoggingFormat::Pretty => {
            fmt::layer().with_writer(writer).pretty().boxed()
        }
        LoggingFormat::Json => fmt::layer().with_writer(writer).json().boxed(),
    }
}

pub fn init(logging_config: &LoggingConfig) -> crate::Result<()> {
    let mut setup_result = Ok(());
    LOGGING_INIT.get_or_init(|| {
        let (console, console_guard) =
            tracing_appender::non_blocking::NonBlockingBuilder::default()
                .buffered_lines_limit(logging_config.buffer_limit)
                .lossy(logging_config.lossy)
                .finish(std::io::stdout());

        let console_layer = new_fmt_layer(logging_config.format, console);

        let mut guards = vec![console_guard];

        let file_layer = if let Some(file_config) = logging_config.file.as_ref()
        {
            let (file_writer, file_guard) =
                tracing_appender::non_blocking::NonBlockingBuilder::default()
                    .buffered_lines_limit(logging_config.buffer_limit)
                    .lossy(logging_config.lossy)
                    .finish(daily(
                        file_config.directory.as_str(),
                        file_config.filename.as_str(),
                    ));
            guards.push(file_guard);
            Some(new_fmt_layer(file_config.format, file_writer))
        } else {
            None
        };

        opentelemetry::global::set_text_map_propagator(
            TraceContextPropagator::new(),
        );

        let otel_provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_span_processor(crate::observability::NoopProcessor)
            .build();

        opentelemetry::global::set_tracer_provider(otel_provider.clone());

        let telemetry_layer = tracing_opentelemetry::layer()
            .with_tracer(otel_provider.tracer("shinespark"));

        let layered = tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new(logging_config.filter.as_str())
            }))
            .with(telemetry_layer)
            .with(console_layer)
            .with(file_layer);

        if let Err(e) = layered.try_init() {
            setup_result = Err(anyhow::Error::new(e)
                .context("failed to init tracing")
                .into());
        }
        guards
    });
    setup_result
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn inner_span() {
        tracing::info!("inside inner span");
        tracing::error!("inside inner span");
    }

    fn default_config() -> LoggingConfig {
        LoggingConfig {
            filter: "debug".into(),
            format: crate::config::LoggingFormat::Compact,
            file: None,
            buffer_limit: 256_000,
            lossy: true,
        }
    }

    #[tokio::test]
    async fn test_logging_basic() {
        let _ = init_tracing(&default_config());
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
        let _ = init_tracing(&default_config());

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
        let config = LoggingConfig {
            filter: "debug".into(),
            format: crate::config::LoggingFormat::Compact,
            file: Some(crate::config::LoggingFileConfig {
                format: crate::config::LoggingFormat::Json,
                directory: temp_dir.into(),
                filename: "test_malt.log".into(),
            }),
            buffer_limit: 256_000,
            lossy: true,
        };

        // Note: Only the first call to init() actually sets the global logger.
        // Subsequent calls are ignored due to OnceLock.
        let _ = init_tracing(&config);

        tracing::info!("test file logging event");

        // We can at least verify the directory exists if this was the first init call
        if std::path::Path::new(temp_dir).exists() {
            println!("Log directory {} exists", temp_dir);
        }
    }
}
