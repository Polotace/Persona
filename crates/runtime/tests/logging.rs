use persona_core::CorrelationId;
use persona_runtime::{LogLevel, LoggerFactory, SafeLogRecord, TracingLoggerFactory};

#[test]
fn safe_log_record_exposes_only_structural_lifecycle_metadata() {
    let correlation_id = CorrelationId::new();
    let record = SafeLogRecord::transition("runtime", "ready", correlation_id);

    assert_eq!(record.component, "runtime");
    assert_eq!(record.transition, "ready");
    assert_eq!(record.correlation_id, correlation_id);
}

#[test]
fn default_tracing_factory_accepts_validated_levels_without_global_setup() {
    let factory = TracingLoggerFactory;

    for level in [
        LogLevel::Error,
        LogLevel::Warn,
        LogLevel::Info,
        LogLevel::Debug,
        LogLevel::Trace,
    ] {
        let logger = factory
            .initialize(level)
            .expect("host-owned tracing logger initializes");
        logger.record(SafeLogRecord::transition(
            "runtime",
            "ready",
            CorrelationId::new(),
        ));
    }
}
