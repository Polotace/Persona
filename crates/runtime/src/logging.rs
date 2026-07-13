use std::sync::Arc;

use persona_core::CorrelationId;

/// A lifecycle log entry containing only structural metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SafeLogRecord {
    pub component: &'static str,
    pub transition: &'static str,
    pub correlation_id: CorrelationId,
}

impl SafeLogRecord {
    /// Creates a lifecycle record without accepting message or payload content.
    pub const fn transition(
        component: &'static str,
        transition: &'static str,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            component,
            transition,
            correlation_id,
        }
    }
}

/// Records privacy-safe runtime lifecycle events.
pub trait RuntimeLogger: Send + Sync {
    /// Records a lifecycle transition using only the fields in [`SafeLogRecord`].
    fn record(&self, record: SafeLogRecord);
}

/// Creates runtime loggers for a configured logging level.
pub trait LoggerFactory: Send + Sync {
    /// Initializes a logger using the requested level.
    fn initialize(&self, log_level: &str) -> Result<Arc<dyn RuntimeLogger>, String>;
}

/// Factory for lifecycle loggers that emit through `tracing`.
#[derive(Default)]
pub struct TracingLoggerFactory;

impl LoggerFactory for TracingLoggerFactory {
    fn initialize(&self, _log_level: &str) -> Result<Arc<dyn RuntimeLogger>, String> {
        Ok(Arc::new(TracingRuntimeLogger))
    }
}

struct TracingRuntimeLogger;

impl RuntimeLogger for TracingRuntimeLogger {
    fn record(&self, record: SafeLogRecord) {
        tracing::info!(
            component = record.component,
            transition = record.transition,
            correlation_id = ?record.correlation_id,
            "runtime lifecycle"
        );
    }
}
