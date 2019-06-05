use std::io;

use atomicwrites;
use chownr;
use failure::Fail;
use serde_json;
use walkdir;

/// Error type returned by all API calls.
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "not found")]
    NotFound,
    #[fail(display = "integrity check failed")]
    IntegrityError,
    #[fail(display = "size check failed")]
    SizeError,
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
    #[fail(display = "{}", _0)]
    Chownr(#[fail(cause)] chownr::Error),
    #[fail(display = "{}", _0)]
    SerdeJson(#[fail(cause)] serde_json::error::Error),
    #[fail(display = "{}", _0)]
    AtomicWrite(#[fail(cause)] atomicwrites::Error<io::Error>),
    #[fail(display = "{}", _0)]
    WalkDir(#[fail(cause)] walkdir::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

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

impl From<atomicwrites::Error<io::Error>> for Error {
    fn from(error: atomicwrites::Error<io::Error>) -> Self {
        Error::AtomicWrite(error)
    }
}

impl From<walkdir::Error> for Error {
    fn from(error: walkdir::Error) -> Self {
        Error::WalkDir(error)
    }
}
