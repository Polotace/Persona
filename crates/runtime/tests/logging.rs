use persona_core::CorrelationId;
use persona_runtime::SafeLogRecord;

#[test]
fn safe_log_record_exposes_only_structural_lifecycle_metadata() {
    let correlation_id = CorrelationId::new();
    let record = SafeLogRecord::transition("runtime", "ready", correlation_id);

    assert_eq!(record.component, "runtime");
    assert_eq!(record.transition, "ready");
    assert_eq!(record.correlation_id, correlation_id);
}
