use std::path::PathBuf;

use ssri::Integrity;
use thiserror::Error;

/// Error type returned by all API calls.
#[derive(Error, Debug)]
pub enum Error {
    /// Returned when an index entry could not be found during
    /// lookup.
    #[error("Entry not found for key {1:?} in cache {0:?}")]
    EntryNotFound(PathBuf, String),
    /// Returned when an integrity check has failed.
    #[error("Integrity check failed.\n\tWanted: {0}\n\tActual: {1}")]
    IntegrityError(Integrity, Integrity),
    /// Returned when a size check has failed.
    #[error("Size check failed.\n\tWanted: {0}\n\tActual: {1}")]
    SizeError(usize, usize),
}
