
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use crate::common::errors::{CliError, CliResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonMessage {
    /// Status request/response
    StatusRequest,
    StatusResponse {
        running: bool,
        pid: Option<u32>,
        uptime_seconds: Option<u64>,
        monitored_repos: Option<usize>,
    },

    /// Repository management
    AddRepositoryRequest {
        path: String,
        name: Option<String>,
    },
    AddRepositoryResponse {
        success: bool,
        error: Option<String>,
    },
    RemoveRepositoryRequest {
        identifier: String,
    },
    RemoveRepositoryResponse {
        success: bool,
        error: Option<String>,
    },

    /// Command execution
    CommandRequest {
        command: String,
        args: Vec<String>,
    },
    CommandResponse {
        success: bool,
        message: String,
        data: Option<String>,
    },

    /// Shutdown request/response
    ShutdownRequest,
    ShutdownResponse {
        success: bool,
    },
}

/// Wrapper for all IPC messages with size information
#[derive(Debug, Serialize, Deserialize)]
pub struct IpcMessage {
    pub size: usize,
    pub payload: DaemonMessage,
}

impl IpcMessage {
    pub fn new(payload: DaemonMessage) -> CliResult<Self> {
        let size = bincode::serialized_size(&payload)
            .map_err(|e| CliError::Ipc(format!("Failed to calculate message size: {}", e)))?;
            
        Ok(Self {
            size: size as usize,
            payload,
        })
    }
}

