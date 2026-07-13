use std::sync::{Arc, Mutex};

use persona_core::{EventError, RuntimeEvent};
use tokio::sync::mpsc;

/// Publishes runtime events to a bounded in-process queue.
#[derive(Clone)]
pub struct EventDispatcher {
    sender: Arc<Mutex<Option<mpsc::Sender<RuntimeEvent>>>>,
}

impl EventDispatcher {
    /// Creates a dispatcher and its sole receiver with the requested queue capacity.
    pub fn bounded(capacity: usize) -> (Self, mpsc::Receiver<RuntimeEvent>) {
        let (sender, receiver) = mpsc::channel(capacity);
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
            .expect("event dispatcher mutex must not be poisoned")
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
            .expect("event dispatcher mutex must not be poisoned")
            .take();
    }
}
