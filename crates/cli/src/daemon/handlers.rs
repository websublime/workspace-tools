
use crate::common::errors::CliResult;
use crate::ipc::DaemonMessage;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct CommandHandler {
    repositories: HashMap<String, PathBuf>,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            repositories: HashMap::new(),
        }
    }

    pub fn repository_count(&self) -> usize {
        self.repositories.len()
    }

    pub fn handle_message(&mut self, message: DaemonMessage) -> CliResult<DaemonMessage> {
        match message {
            DaemonMessage::AddRepositoryRequest { path, name } => {
                self.handle_add_repository(path, name)
            }
            DaemonMessage::RemoveRepositoryRequest { identifier } => {
                self.handle_remove_repository(identifier)
            }
            DaemonMessage::CommandRequest { command, args } => {
                self.handle_command(command, args)
            }
            _ => Ok(DaemonMessage::CommandResponse {
                success: false,
                message: "Unsupported message type".to_string(),
                data: None,
            }),
        }
    }

    fn handle_add_repository(&mut self, path: String, name: Option<String>) -> CliResult<DaemonMessage> {
        let path_buf = PathBuf::from(&path);
        
        // Validate repository path
        if !path_buf.exists() {
            return Ok(DaemonMessage::AddRepositoryResponse {
                success: false,
                error: Some(format!("Repository path does not exist: {}", path)),
            });
        }

        // Add to repositories
        let identifier = name.unwrap_or_else(|| path.clone());
        self.repositories.insert(identifier, path_buf);

        Ok(DaemonMessage::AddRepositoryResponse {
            success: true,
            error: None,
        })
    }

    fn handle_remove_repository(&mut self, identifier: String) -> CliResult<DaemonMessage> {
        match self.repositories.remove(&identifier) {
            Some(_) => Ok(DaemonMessage::RemoveRepositoryResponse {
                success: true,
                error: None,
            }),
            None => Ok(DaemonMessage::RemoveRepositoryResponse {
                success: false,
                error: Some(format!("Repository not found: {}", identifier)),
            }),
        }
    }

    fn handle_command(&self, command: String, args: Vec<String>) -> CliResult<DaemonMessage> {
        match command.as_str() {
            "list-repos" => {
                let repos: Vec<_> = self.repositories.keys().cloned().collect();
                Ok(DaemonMessage::CommandResponse {
                    success: true,
                    message: format!("Found {} repositories", repos.len()),
                    data: Some(repos.join("\n")),
                })
            }
            _ => Ok(DaemonMessage::CommandResponse {
                success: false,
                message: format!("Unknown command: {}", command),
                data: None,
            }),
        }
    }
}

