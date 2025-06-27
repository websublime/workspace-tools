//! Event system for component communication
//!
//! This module provides a decoupled event system that allows components to communicate
//! without direct dependencies. Components can emit events and subscribe to events
//! from other components without knowing about their implementation.
//!
//! The event system follows the Observer pattern and provides:
//! - Type-safe event definitions
//! - Async event handling
//! - Event filtering and routing
//! - Lifecycle management for subscriptions

pub mod bus;
pub mod handlers;
pub mod types;

pub use bus::{EventBus, EventSubscription};
pub use handlers::{EventHandler, AsyncEventHandler};
pub use types::{MonorepoEvent, EventContext, EventPriority};

use crate::error::Result;

/// Event system trait for components that can emit and receive events
pub trait EventEmitter {
    /// Emit an event to the event bus
    fn emit_event(&self, event: MonorepoEvent) -> Result<()>;
}

/// Event system trait for components that can subscribe to events
pub trait EventSubscriber {
    /// Subscribe to specific event types
    fn subscribe_to_events(&mut self, bus: &mut EventBus) -> Result<()>;
}

/// Event coordinator trait for managing component lifecycle with events
pub trait EventCoordinator {
    /// Initialize event system integration
    fn initialize_events(&mut self, bus: &mut EventBus) -> Result<()>;
    
    /// Shutdown and cleanup event subscriptions
    fn shutdown_events(&mut self) -> Result<()>;
}