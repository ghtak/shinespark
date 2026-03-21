use std::sync::OnceLock;

use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

use crate::config::{TraceConfig, TraceFormat};

static LOGGING_INIT: OnceLock<Vec<WorkerGuard>> = OnceLock::new();

fn build_layer<S>(
    filter: &str,
    format: TraceFormat,
    writer: tracing_appender::non_blocking::NonBlocking,
) -> Box<dyn tracing_subscriber::Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
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
    let mut result = Ok(());
    LOGGING_INIT.get_or_init(|| {
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
        if let Err(e) = tracing_subscriber::registry().with(layers).try_init() {
            result = Err(anyhow::anyhow!(e).into());
        }
        guards
    });
    result
}
