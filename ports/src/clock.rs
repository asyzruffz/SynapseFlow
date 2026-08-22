use std::time::SystemTime;

/// Supplies time without coupling domain/application code to a runtime.
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}
