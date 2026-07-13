use std::time::SystemTime;

use persona_core::{CorrelationId, OwnerId, RuntimeEvent, RuntimeEventKind, SchemaVersion};

#[test]
fn correlation_id_default_creates_a_value() {
    let correlation_id = CorrelationId::default();

    assert_ne!(correlation_id.as_uuid(), uuid::Uuid::nil());
}

#[test]
fn owner_id_rejects_blank_values() {
    for value in ["", " \t\n "] {
        assert!(
            OwnerId::try_from(value).is_err(),
            "expected {value:?} to be rejected"
        );
    }
}

#[test]
fn runtime_event_preserves_contract_metadata_and_records_occurrence_time() {
    let correlation_id = CorrelationId::new();
    let before = SystemTime::now();
    let event = RuntimeEvent::ready(SchemaVersion::V1, correlation_id);
    let after = SystemTime::now();

    assert_eq!(event.schema_version(), SchemaVersion::V1);
    assert_eq!(event.correlation_id(), correlation_id);
    assert!(before <= event.occurred_at());
    assert!(event.occurred_at() <= after);
    assert_eq!(event.kind(), RuntimeEventKind::RuntimeReady);
}
