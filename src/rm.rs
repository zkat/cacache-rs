//! Functions for removing things from the cache.
use std::fs;
use std::path::Path;

use ssri::Integrity;

use crate::content::rm;
use crate::errors::Error;
use crate::index;

pub fn entry<P: AsRef<Path>>(cache: P, key: &str) -> Result<(), Error> {
    index::delete(cache.as_ref(), &key)
}

pub fn content<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<(), Error> {
    rm::rm(cache.as_ref(), &sri)
}

pub fn all<P: AsRef<Path>>(cache: P) -> Result<(), Error> {
    for entry in cache.as_ref().read_dir()? {
        if let Ok(entry) = entry {
            fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}
