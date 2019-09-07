// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

#[cfg(unix)]
use chownr;
use failure::Fail;
use serde_json;
use tempfile;
use walkdir;

/// Error type returned by all API calls.
#[derive(Fail, Debug)]
pub enum Error {
    /// Returned when an index or content entry could not be found during
    /// lookup.
    #[fail(display = "not found")]
    NotFound,
    /// Returned when an integrity check has failed.
    #[fail(display = "integrity check failed")]
    IntegrityError,
    /// Returned when a size check has failed.
    #[fail(display = "size check failed")]
    SizeError,
    /// Returned when there's an std::io::Error.
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
    /// Returned when there's an error with changing uid/gid on an entry.
    #[fail(display = "{}", _0)]
    #[cfg(unix)]
    Chownr(#[fail(cause)] chownr::Error),
    /// Returned when there's an issue with metadata (de)serialization.
    #[fail(display = "{}", _0)]
    SerdeJson(#[fail(cause)] serde_json::error::Error),
    /// Returned when a content entry could not be moved to its final
    /// destination.
    #[fail(display = "{}", _0)]
    PersistError(#[fail(cause)] tempfile::PersistError),
    /// Returned when something went wrong while traversing the index during
    /// `cacache::ls`.
    #[fail(display = "{}", _0)]
    WalkDir(#[fail(cause)] walkdir::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

#[cfg(unix)]
impl From<chownr::Error> for Error {
    fn from(error: chownr::Error) -> Self {
        Error::Chownr(error)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Self {
        Error::SerdeJson(error)
    }
}

impl From<tempfile::PersistError> for Error {
    fn from(error: tempfile::PersistError) -> Self {
        Error::PersistError(error)
    }
}

impl From<walkdir::Error> for Error {
    fn from(error: walkdir::Error) -> Self {
        Error::WalkDir(error)
    }
}
