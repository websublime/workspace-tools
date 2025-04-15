//! Daemon service for workspace-cli
//!
//! The daemon service runs in the background and monitors repositories,
//! providing real-time updates and handling events.

mod event;
mod ipc;
mod service;
mod status;
mod watcher;

pub use event::{
    error_event, info_event, success_event, warning_event, Event, EventHandler, EventType,
};
pub use ipc::{IpcClient, IpcMessage, IpcResponse, IpcServer};
pub use service::{DaemonService, ServiceCommand, ServiceConfig};
pub use status::{DaemonStatus, RepositoryStatus, StatusInfo};
pub use watcher::RepositoryWatcher;
