use thiserror::Error;

/// Error type returned by all API calls.
#[derive(Error, Debug)]
pub enum Error {
    /// Returned when an index or content entry could not be found during
    /// lookup.
    #[error("not found")]
    NotFound,
    /// Returned when an integrity check has failed.
    #[error("integrity check failed")]
    IntegrityError,
    /// Returned when a size check has failed.
    #[error("size check failed")]
    SizeError,
}
