// =============================================================================
// REACTOR Event Bus System
// =============================================================================
// Provides a thread-safe, decoupled event publisher/subscriber pattern.
// Any component or user code can register observers for any event type,
// and emit events.
// =============================================================================

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// A thread-safe, generic event bus that distributes events to registered observers.
#[derive(Clone)]
pub struct EventBus {
    senders: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl EventBus {
    /// Create a new EventBus
    pub fn new() -> Self {
        Self {
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register to receive events of type `T`. Returns an `Observer<T>`.
    pub fn register<T: Clone + Send + Sync + 'static>(&self) -> Observer<T> {
        let (tx, rx) = unbounded_channel::<T>();
        let mut senders = self.senders.lock().unwrap();
        let type_id = TypeId::of::<T>();

        let list = senders
            .entry(type_id)
            .or_insert_with(|| Box::new(Vec::<UnboundedSender<T>>::new()));

        if let Some(vec) = list.downcast_mut::<Vec<UnboundedSender<T>>>() {
            vec.push(tx);
        }

        Observer::new(rx)
    }

    /// Emit an event of type `T` to all registered observers.
    pub fn emit<T: Clone + Send + Sync + 'static>(&self, event: T) {
        let mut senders = self.senders.lock().unwrap();
        let type_id = TypeId::of::<T>();

        if let Some(list) = senders.get_mut(&type_id) {
            if let Some(vec) = list.downcast_mut::<Vec<UnboundedSender<T>>>() {
                // Keep only active senders (remove disconnected receivers)
                vec.retain(|tx| tx.send(event.clone()).is_ok());
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// An observer that receives events of type `T` from the EventBus.
pub struct Observer<T> {
    rx: UnboundedReceiver<T>,
}

impl<T> Observer<T> {
    pub fn new(rx: UnboundedReceiver<T>) -> Self {
        Self { rx }
    }

    /// Poll for the next pending event without blocking.
    pub fn poll(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }

    /// Drain all pending events in the queue.
    pub fn drain(&mut self) -> Vec<T> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            events.push(event);
        }
        events
    }

    /// Await the next event asynchronously.
    pub async fn recv(&mut self) -> Option<T> {
        self.rx.recv().await
    }
}
