use napi::bindgen_prelude::*;
use sublime_package_tools::VersionError;

#[derive(Debug)]
#[allow(dead_code)]
pub enum JsVersionError {
    /// Error originating from the Node-API layer
    NapiError(Error),
    /// Error originating from version
    VersionError(VersionError),
}

impl From<VersionError> for JsVersionError {
    fn from(err: VersionError) -> Self {
        JsVersionError::VersionError(err)
    }
}

impl AsRef<str> for JsVersionError {
    fn as_ref(&self) -> &str {
        match self {
            JsVersionError::NapiError(e) => e.status.as_ref(),
            JsVersionError::VersionError(e) => e.as_ref(),
        }
    }
}

pub(crate) fn version_format_napi_error(err: VersionError) -> Error<JsVersionError> {
    Error::new(err.clone().into(), err.to_string())
}
