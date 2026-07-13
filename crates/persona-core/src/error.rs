#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("owner identifier must not be blank")]
    BlankOwnerId,
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("event queue is full")]
    QueueFull,
    #[error("event dispatcher is closed")]
    Closed,
}

#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("invalid lifecycle transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: crate::RuntimeState,
        to: crate::RuntimeState,
    },
}
