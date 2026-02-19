use opentelemetry::Context;
use opentelemetry::trace::TraceResult;
use opentelemetry::trace::{TraceContextExt, TraceId};
use opentelemetry_sdk::export::trace::SpanData;
use opentelemetry_sdk::trace::Span;
use opentelemetry_sdk::trace::SpanProcessor;
use rand::Rng;
use rand::rng;
use tracing_opentelemetry::OpenTelemetrySpanExt;

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
