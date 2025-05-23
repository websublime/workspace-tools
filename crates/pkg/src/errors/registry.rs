use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Failed to support url: {0}")]
    UrlNotSupported(String),
    #[error("Registry not found: {0}")]
    UrlNotFound(String),
    #[error("Failed to read npmrc file at: {path}")]
    NpmRcFailure {
        path: String,
        #[source]
        error: io::Error,
    },
}

impl From<io::Error> for RegistryError {
    fn from(error: io::Error) -> Self {
        RegistryError::NpmRcFailure { path: error.to_string(), error }
    }
}

impl AsRef<str> for RegistryError {
    fn as_ref(&self) -> &str {
        match self {
            RegistryError::UrlNotSupported(_) => "UrlNotSupported",
            RegistryError::UrlNotFound(_) => "UrlNotFound",
            RegistryError::NpmRcFailure { path: _, error: _ } => "NpmRcFailure",
        }
    }
}

impl Clone for RegistryError {
    fn clone(&self) -> Self {
        match self {
            RegistryError::NpmRcFailure { path, error } => {
                RegistryError::NpmRcFailure {
                    path: path.clone(),
                    // Create a new io::Error with the same kind and message
                    error: io::Error::new(error.kind(), error.to_string()),
                }
            }
            _ => self.clone(),
        }
    }
}
