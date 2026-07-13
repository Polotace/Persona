use std::sync::Arc;

use persona_core::CorrelationId;

use crate::LogLevel;

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
    fn initialize(&self, log_level: LogLevel) -> Result<Arc<dyn RuntimeLogger>, String>;
}

/// Factory for lifecycle loggers that emit through an application-provided `tracing` subscriber.
///
/// This factory deliberately does not configure a global subscriber. Application hosts own that
/// process-wide decision and can replace this factory when they require another logging backend.
#[derive(Default)]
pub struct TracingLoggerFactory;

impl LoggerFactory for TracingLoggerFactory {
    fn initialize(&self, log_level: LogLevel) -> Result<Arc<dyn RuntimeLogger>, String> {
        Ok(Arc::new(TracingRuntimeLogger { log_level }))
    }
}

struct TracingRuntimeLogger {
    log_level: LogLevel,
}

impl RuntimeLogger for TracingRuntimeLogger {
    fn record(&self, record: SafeLogRecord) {
        match self.log_level {
            LogLevel::Error => tracing::error!(
                component = record.component,
                transition = record.transition,
                correlation_id = ?record.correlation_id,
                "runtime lifecycle"
            ),
            LogLevel::Warn => tracing::warn!(
                component = record.component,
                transition = record.transition,
                correlation_id = ?record.correlation_id,
                "runtime lifecycle"
            ),
            LogLevel::Info => tracing::info!(
                component = record.component,
                transition = record.transition,
                correlation_id = ?record.correlation_id,
                "runtime lifecycle"
            ),
            LogLevel::Debug => tracing::debug!(
                component = record.component,
                transition = record.transition,
                correlation_id = ?record.correlation_id,
                "runtime lifecycle"
            ),
            LogLevel::Trace => tracing::trace!(
                component = record.component,
                transition = record.transition,
                correlation_id = ?record.correlation_id,
                "runtime lifecycle"
            ),
        }
    }
}
