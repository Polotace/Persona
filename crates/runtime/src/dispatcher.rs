use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use persona_core::{EventError, RuntimeEvent};
use tokio::sync::mpsc;

const LIFECYCLE_EVENT_COUNT: usize = 5;

/// Publishes runtime events to a bounded in-process queue.
#[derive(Clone)]
pub struct EventDispatcher {
    sender: Arc<Mutex<Option<mpsc::Sender<RuntimeEvent>>>>,
}

impl EventDispatcher {
    /// Creates a dispatcher and its sole receiver with enough bounded capacity for lifecycle facts.
    ///
    /// A runtime always emits three startup and two shutdown lifecycle events before a consumer
    /// can necessarily drain the queue. The configured capacity is therefore raised to this
    /// small, fixed minimum; the queue remains bounded and still reports backpressure once full.
    pub fn bounded(capacity: NonZeroUsize) -> (Self, mpsc::Receiver<RuntimeEvent>) {
        let (sender, receiver) = mpsc::channel(capacity.get().max(LIFECYCLE_EVENT_COUNT));
        let dispatcher = Self {
            sender: Arc::new(Mutex::new(Some(sender))),
        };

        (dispatcher, receiver)
    }

    /// Attempts to enqueue an event without waiting for available capacity.
    pub fn try_publish(&self, event: RuntimeEvent) -> Result<(), EventError> {
        let sender = self
            .sender
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .ok_or(EventError::Closed)?;

        sender.try_send(event).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => EventError::QueueFull,
            mpsc::error::TrySendError::Closed(_) => EventError::Closed,
        })
    }

    /// Closes the dispatcher so subsequent publish attempts fail immediately.
    pub fn close(&self) {
        self.sender
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
    }
}
