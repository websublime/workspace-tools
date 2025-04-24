use std::io;
use std::time::Duration;
use crate::common::errors::CliResult;
use super::messages::IpcMessage;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
pub const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub trait IpcTransport: Send + Sync {
    fn connect(&self) -> CliResult<Box<dyn IpcConnection>>;
    fn bind(&self) -> CliResult<Box<dyn IpcListener>>;
}

pub trait IpcConnection: Send {
    fn send_message(&mut self, message: &IpcMessage) -> CliResult<()>;
    fn receive_message(&mut self) -> CliResult<IpcMessage>;
    fn set_read_timeout(&mut self, timeout: Option<Duration>) -> CliResult<()>;
    fn set_write_timeout(&mut self, timeout: Option<Duration>) -> CliResult<()>;
}

pub trait IpcListener: Send {
    fn accept(&mut self) -> CliResult<Box<dyn IpcConnection>>;
    fn set_nonblocking(&mut self, nonblocking: bool) -> CliResult<()>;
}

// Helper functions for reading/writing with timeout
pub(crate) fn read_with_timeout(
    read: &mut impl io::Read,
    buf: &mut [u8],
    timeout: Duration,
) -> io::Result<()> {
    let start = std::time::Instant::now();
    let mut bytes_read = 0;

    while bytes_read < buf.len() {
        if start.elapsed() > timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Read operation timed out",
            ));
        }

        match read.read(&mut buf[bytes_read..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Connection closed",
                ));
            }
            Ok(n) => {
                bytes_read += n;
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

pub(crate) fn write_with_timeout(
    write: &mut impl io::Write,
    buf: &[u8],
    timeout: Duration,
) -> io::Result<()> {
    let start = std::time::Instant::now();
    let mut bytes_written = 0;

    while bytes_written < buf.len() {
        if start.elapsed() > timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Write operation timed out",
            ));
        }

        match write.write(&buf[bytes_written..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "Failed to write data",
                ));
            }
            Ok(n) => {
                bytes_written += n;
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

