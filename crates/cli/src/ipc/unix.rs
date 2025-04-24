
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::Duration;

use crate::common::errors::{CliError, CliResult};
use crate::ipc::messages::IpcMessage;
use super::transport::{IpcConnection, IpcListener, IpcTransport, read_with_timeout, write_with_timeout, MAX_MESSAGE_SIZE};

pub struct UnixSocketTransport {
    socket_path: std::path::PathBuf,
}

impl UnixSocketTransport {
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }
}

impl IpcTransport for UnixSocketTransport {
    fn connect(&self) -> CliResult<Box<dyn IpcConnection>> {
        let stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| CliError::Io(e))?;
        
        Ok(Box::new(UnixSocketConnection { stream }))
    }

    fn bind(&self) -> CliResult<Box<dyn IpcListener>> {
        // Remove socket file if it already exists
        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)
                .map_err(|e| CliError::Io(e))?;
        }
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.socket_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CliError::Io(e))?;
        }
        
        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| CliError::Io(e))?;
        
        // Set socket permissions to be world accessible
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.socket_path, fs::Permissions::from_mode(0o777))
                .map_err(|e| CliError::Io(e))?;
        }
        
        Ok(Box::new(UnixSocketListener { listener }))
    }
}

pub struct UnixSocketListener {
    listener: UnixListener,
}

impl IpcListener for UnixSocketListener {
    fn accept(&mut self) -> CliResult<Box<dyn IpcConnection>> {
        match self.listener.accept() {
            Ok((stream, _)) => Ok(Box::new(UnixSocketConnection { stream })),
            Err(e) => Err(CliError::Io(e)),
        }
    }

    fn set_nonblocking(&mut self, nonblocking: bool) -> CliResult<()> {
        self.listener.set_nonblocking(nonblocking)
            .map_err(|e| CliError::Io(e))
    }
}

pub struct UnixSocketConnection {
    stream: UnixStream,
}

impl UnixSocketConnection {
    fn set_timeout(&self, duration: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(duration)?;
        self.stream.set_write_timeout(duration)
    }
}

impl IpcConnection for UnixSocketConnection {
    fn send_message(&mut self, message: &IpcMessage) -> CliResult<()> {
        // Serialize message
        let message_bytes = bincode::serialize(message)
            .map_err(|e| CliError::Ipc(format!("Failed to serialize message: {}", e)))?;
        
        // Send length prefix
        let len = message_bytes.len() as u32;
        let len_bytes = len.to_be_bytes();
        
        write_with_timeout(&mut self.stream, &len_bytes, Duration::from_secs(5))
            .map_err(|e| CliError::Io(e))?;
        
        // Send message
        write_with_timeout(&mut self.stream, &message_bytes, Duration::from_secs(5))
            .map_err(|e| CliError::Io(e))?;
        
        Ok(())
    }

    fn receive_message(&mut self) -> CliResult<IpcMessage> {
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        read_with_timeout(&mut self.stream, &mut len_bytes, Duration::from_secs(5))
            .map_err(|e| CliError::Io(e))?;
        
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        // Check size limit
        if len > MAX_MESSAGE_SIZE {
            return Err(CliError::Ipc(format!("Message too large: {} bytes", len)));
        }
        
        // Read message
        let mut message_bytes = vec![0u8; len];
        read_with_timeout(&mut self.stream, &mut message_bytes, Duration::from_secs(5))
            .map_err(|e| CliError::Io(e))?;
        
        // Deserialize message
        let message = bincode::deserialize(&message_bytes)
            .map_err(|e| CliError::Ipc(format!("Failed to deserialize message: {}", e)))?;
        
        Ok(message)
    }
    
    fn set_read_timeout(&mut self, timeout: Option<Duration>) -> CliResult<()> {
        self.stream.set_read_timeout(timeout)
            .map_err(|e| CliError::Io(e))
    }

    fn set_write_timeout(&mut self, timeout: Option<Duration>) -> CliResult<()> {
        self.stream.set_write_timeout(timeout)
            .map_err(|e| CliError::Io(e))
    }
}

