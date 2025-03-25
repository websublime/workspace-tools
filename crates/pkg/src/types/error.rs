use thiserror::Error;

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Failed to parse version: {message}")]
    Parse {
        #[source]
        error: semver::Error,
        message: String,
    },
}

impl From<semver::Error> for VersionError {
    fn from(error: semver::Error) -> Self {
        VersionError::Parse { message: error.to_string(), error }
    }
}

impl Clone for VersionError {
    fn clone(&self) -> Self {
        match self {
            VersionError::Parse { message, .. } => {
                // Create a new semver::Error
                let error = semver::Version::parse("invalid-version").unwrap_err();
                VersionError::Parse { error, message: message.clone() }
            }
        }
    }
}

impl AsRef<str> for VersionError {
    fn as_ref(&self) -> &str {
        match self {
            VersionError::Parse { error: _, message: _ } => "VersionErrorParse",
        }
    }
}
