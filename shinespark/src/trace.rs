use std::sync::OnceLock;

use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::daily;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

use crate::config::{TraceConfig, TraceFormat};

static LOGGING_GUARDS: OnceLock<Vec<WorkerGuard>> = OnceLock::new();

fn build_layer<S, W>(
    filter: &str,
    format: TraceFormat,
    writer: W,
) -> Box<dyn tracing_subscriber::Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    W: for<'writer> tracing_subscriber::fmt::MakeWriter<'writer> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));
    let layer = fmt::layer().with_writer(writer);
    match format {
        TraceFormat::Full => layer.with_filter(env_filter).boxed(),
        TraceFormat::Compact => layer.compact().with_filter(env_filter).boxed(),
        TraceFormat::Pretty => layer.pretty().with_filter(env_filter).boxed(),
        TraceFormat::Json => layer.json().with_filter(env_filter).boxed(),
    }
}

pub fn init(trace_config: &TraceConfig) -> crate::Result<()> {
    // try_init() natively guards against multiple initializations and returns an error
    let mut guards = Vec::new();
    let mut layers = Vec::new();

    if let Some(console) = trace_config.console.as_ref() {
        let (w, g) = tracing_appender::non_blocking::NonBlockingBuilder::default()
            .buffered_lines_limit(console.buffered_lines_limit)
            .lossy(false)
            .finish(std::io::stderr());
        layers.push(build_layer(console.filter.as_str(), console.format, w));
        guards.push(g);
    }

    if let Some(file) = trace_config.file.as_ref() {
        let (w, g) = tracing_appender::non_blocking::NonBlockingBuilder::default()
            .buffered_lines_limit(file.buffered_lines_limit)
            .lossy(false)
            .finish(daily(file.directory.as_str(), file.prefix.as_str()));
        layers.push(build_layer(file.filter.as_str(), file.format, w));
        guards.push(g);
    }

    // Initialize the global subscriber
    tracing_subscriber::registry()
        .with(layers)
        .try_init()
        .map_err(|e| anyhow::anyhow!(e))?;

    // Store guards to keep the non-blocking appender threads alive
    LOGGING_GUARDS.get_or_init(|| guards);

    Ok(())
}
