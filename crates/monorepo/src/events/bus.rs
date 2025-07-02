//! Event bus implementation for component communication
//!
//! The event bus provides a central coordination point for all events in the system.
//! Components can emit events and subscribe to events without knowing about each other.

use super::handlers::AsyncEventHandlerWrapper;
use super::{AsyncEventHandler, EventPriority, MonorepoEvent};
use crate::error::{Error, Result};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Event subscription handle
#[derive(Debug, Clone)]
pub struct EventSubscription {
    /// Unique subscription ID
    pub id: Uuid,

    /// Event filter pattern
    pub filter: EventFilter,

    /// Handler for processing events
    pub handler: Arc<AsyncEventHandlerWrapper>,
}

/// Event filtering criteria
#[derive(Debug, Clone)]
pub enum EventFilter {
    /// Accept all events
    All,

    /// Filter by event type
    ByType(EventTypeFilter),

    /// Filter by source component
    BySource(String),

    /// Filter by priority level
    ByPriority(EventPriority),

    /// Combine multiple filters with AND logic
    And(Vec<EventFilter>),

    /// Combine multiple filters with OR logic
    Or(Vec<EventFilter>),

    /// Custom predicate function
    Custom(fn(&MonorepoEvent) -> bool),
}

/// Event type filtering options
#[derive(Debug, Clone)]
pub enum EventTypeFilter {
    /// Config events only
    Config,
    /// Task events only
    Task,
    /// Changeset events only
    Changeset,
    /// Hook events only
    Hook,
    /// Package events only
    Package,
    /// File system events only
    FileSystem,
    /// Workflow events only
    Workflow,
}

/// Priority queue item for event processing
#[derive(Debug)]
struct QueuedEvent {
    event: MonorepoEvent,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl PartialEq for QueuedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.priority() == other.event.priority() && self.timestamp == other.timestamp
    }
}

impl Eq for QueuedEvent {}

impl PartialOrd for QueuedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority events come first, then by timestamp (older first)
        self.event
            .priority()
            .cmp(&other.event.priority())
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

/// Central event bus for coordinating component communication
pub struct EventBus {
    /// Active subscriptions
    subscriptions: RwLock<HashMap<Uuid, EventSubscription>>,

    /// Event broadcast channel
    sender: broadcast::Sender<MonorepoEvent>,

    /// Priority queue for event processing
    event_queue: RwLock<BinaryHeap<QueuedEvent>>,

    /// Statistics tracking
    stats: RwLock<EventBusStats>,
}

/// Event bus statistics
#[derive(Debug, Default)]
pub struct EventBusStats {
    /// Total events emitted
    pub events_emitted: u64,

    /// Total events processed
    pub events_processed: u64,

    /// Events by type
    pub events_by_type: HashMap<String, u64>,

    /// Events by priority
    pub events_by_priority: HashMap<EventPriority, u64>,

    /// Active subscriptions count
    pub active_subscriptions: usize,
}

impl EventBus {
    /// Create a new event bus
    #[must_use]
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000); // Buffer up to 1000 events

        Self {
            subscriptions: RwLock::new(HashMap::new()),
            sender,
            event_queue: RwLock::new(BinaryHeap::new()),
            stats: RwLock::new(EventBusStats::default()),
        }
    }

    /// Emit an event to all subscribers
    pub async fn emit(&self, event: MonorepoEvent) -> Result<()> {
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.events_emitted += 1;

            let event_type = Self::get_event_type_name(&event);
            *stats.events_by_type.entry(event_type).or_insert(0) += 1;
            *stats.events_by_priority.entry(event.priority()).or_insert(0) += 1;
        }

        // Add to priority queue for ordered processing
        {
            let mut queue = self.event_queue.write().await;
            queue.push(QueuedEvent { timestamp: event.context().timestamp, event: event.clone() });
        }

        // Broadcast to all listeners
        if let Err(e) = self.sender.send(event) {
            return Err(Error::config(format!("Failed to emit event: {e}")));
        }

        Ok(())
    }

    /// Subscribe to events with a handler
    pub async fn subscribe(
        &self,
        filter: EventFilter,
        handler: Arc<AsyncEventHandlerWrapper>,
    ) -> Result<Uuid> {
        let subscription_id = Uuid::new_v4();
        let subscription = EventSubscription { id: subscription_id, filter, handler };

        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id, subscription);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.active_subscriptions = self.subscriptions.read().await.len();
        }

        Ok(subscription_id)
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: Uuid) -> Result<bool> {
        let removed = {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(&subscription_id).is_some()
        };

        if removed {
            let mut stats = self.stats.write().await;
            stats.active_subscriptions = self.subscriptions.read().await.len();
        }

        Ok(removed)
    }

    /// Process events from the priority queue
    pub async fn process_events(&self, batch_size: usize) -> Result<usize> {
        let events_to_process = {
            let mut queue = self.event_queue.write().await;
            let mut events = Vec::new();

            for _ in 0..batch_size {
                if let Some(queued_event) = queue.pop() {
                    events.push(queued_event.event);
                } else {
                    break;
                }
            }

            events
        };

        let processed_count = events_to_process.len();

        for event in events_to_process {
            self.dispatch_event(&event).await?;
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.events_processed += processed_count as u64;
        }

        Ok(processed_count)
    }

    /// Create a receiver for listening to events
    pub fn create_receiver(&self) -> broadcast::Receiver<MonorepoEvent> {
        self.sender.subscribe()
    }

    /// Get current event bus statistics
    pub async fn get_stats(&self) -> EventBusStats {
        let stats = self.stats.read().await;
        EventBusStats {
            events_emitted: stats.events_emitted,
            events_processed: stats.events_processed,
            events_by_type: stats.events_by_type.clone(),
            events_by_priority: stats.events_by_priority.clone(),
            active_subscriptions: stats.active_subscriptions,
        }
    }

    /// Get pending events count
    pub async fn pending_events_count(&self) -> usize {
        self.event_queue.read().await.len()
    }

    // Private helper methods

    /// Dispatch event to matching subscribers
    async fn dispatch_event(&self, event: &MonorepoEvent) -> Result<()> {
        let subscriptions = self.subscriptions.read().await;

        for subscription in subscriptions.values() {
            if Self::event_matches_filter(event, &subscription.filter) {
                if let Err(e) = subscription.handler.handle_event(event.clone()).await {
                    // Log error but continue processing other handlers
                    log::error!("Event handler failed for subscription {}: {}", subscription.id, e);
                }
            }
        }

        Ok(())
    }

    /// Check if an event matches a filter
    fn event_matches_filter(event: &MonorepoEvent, filter: &EventFilter) -> bool {
        match filter {
            EventFilter::All => true,

            EventFilter::ByType(type_filter) => matches!(
                (event, type_filter),
                (MonorepoEvent::Config(_), EventTypeFilter::Config)
                    | (MonorepoEvent::Task(_), EventTypeFilter::Task)
                    | (MonorepoEvent::Changeset(_), EventTypeFilter::Changeset)
                    | (MonorepoEvent::Hook(_), EventTypeFilter::Hook)
                    | (MonorepoEvent::Package(_), EventTypeFilter::Package)
                    | (MonorepoEvent::FileSystem(_), EventTypeFilter::FileSystem)
                    | (MonorepoEvent::Workflow(_), EventTypeFilter::Workflow)
            ),

            EventFilter::BySource(source) => event.source() == source,

            EventFilter::ByPriority(priority) => event.priority() >= *priority,

            EventFilter::And(filters) => {
                filters.iter().all(|f| Self::event_matches_filter(event, f))
            }

            EventFilter::Or(filters) => filters.iter().any(|f| Self::event_matches_filter(event, f)),

            EventFilter::Custom(predicate) => predicate(event),
        }
    }

    /// Get event type name for statistics
    fn get_event_type_name(event: &MonorepoEvent) -> String {
        match event {
            MonorepoEvent::Config(_) => "Config".to_string(),
            MonorepoEvent::Task(_) => "Task".to_string(),
            MonorepoEvent::Changeset(_) => "Changeset".to_string(),
            MonorepoEvent::Hook(_) => "Hook".to_string(),
            MonorepoEvent::Package(_) => "Package".to_string(),
            MonorepoEvent::FileSystem(_) => "FileSystem".to_string(),
            MonorepoEvent::Workflow(_) => "Workflow".to_string(),
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventFilter {
    /// Create a filter for specific event types
    #[must_use]
    pub fn for_type(event_type: EventTypeFilter) -> Self {
        Self::ByType(event_type)
    }

    /// Create a filter for specific source component
    #[must_use]
    pub fn for_source(source: impl Into<String>) -> Self {
        Self::BySource(source.into())
    }

    /// Create a filter for minimum priority level
    #[must_use]
    pub fn for_priority(priority: EventPriority) -> Self {
        Self::ByPriority(priority)
    }

    /// Combine filters with AND logic
    #[must_use]
    pub fn and(filters: Vec<Self>) -> Self {
        Self::And(filters)
    }

    /// Combine filters with OR logic
    #[must_use]
    pub fn or(filters: Vec<Self>) -> Self {
        Self::Or(filters)
    }
}
