use persona_core::{CorrelationId, EventError, RuntimeEvent, SchemaVersion};
use persona_runtime::EventDispatcher;

#[tokio::test]
async fn dispatcher_enforces_capacity_and_reports_closure() {
    let (dispatcher, mut receiver) = EventDispatcher::bounded(1);
    let event = RuntimeEvent::ready(SchemaVersion::V1, CorrelationId::new());

    dispatcher
        .try_publish(event)
        .expect("the first event should fit in the queue");

    assert!(matches!(
        dispatcher.try_publish(event),
        Err(EventError::QueueFull)
    ));
    assert_eq!(receiver.recv().await, Some(event));

    dispatcher.close();

    assert!(matches!(
        dispatcher.try_publish(event),
        Err(EventError::Closed)
    ));
}
