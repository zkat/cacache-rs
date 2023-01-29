use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

/// Error type returned by all API calls.
#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    /// Returned when an index entry could not be found during
    /// lookup.
    #[error("Entry not found for key {1:?} in cache {0:?}")]
    #[diagnostic(code(cacache::entry_not_found), url(docsrs))]
    EntryNotFound(PathBuf, String),

    /// Returned when a size check has failed.
    #[error("Size check failed.\n\tWanted: {0}\n\tActual: {1}")]
    #[diagnostic(code(cacache::size_mismatch), url(docsrs))]
    SizeMismatch(usize, usize),

    /// Returned when a general IO error has occurred.
    #[error("{1}")]
    #[diagnostic(code(cacache::io_error), url(docsrs))]
    IoError(#[source] std::io::Error, String),

    /// Returned when a general serde error has occurred.
    #[error("{1}")]
    #[diagnostic(code(cacache::serde_error), url(docsrs))]
    SerdeError(#[source] serde_json::Error, String),

    /// Returned when an integrity check has failed.
    #[error(transparent)]
    #[diagnostic(code(cacache::integrity_error), url(docsrs))]
    IntegrityError(#[from] ssri::Error),
}

/// The result type returned by calls to this library
pub type Result<T> = std::result::Result<T, Error>;

pub trait IoErrorExt<T> {
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T>;
}

impl<T> IoErrorExt<T> for std::result::Result<T, std::io::Error> {
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(Error::IoError(e, f())),
        }
    }
}

impl<T> IoErrorExt<T> for std::result::Result<T, serde_json::Error> {
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(Error::SerdeError(e, f())),
        }
    }
}

pub fn io_error(err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err)
}
