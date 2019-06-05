//! Functions for reading from cache.
use std::path::Path;

use ssri::Integrity;

use crate::content::read;
use crate::errors::Error;
use crate::index::{self, Entry};

pub fn read(cache: &Path, key: String) -> Result<Vec<u8>, Error> {
    if let Some(entry) = index::find(&cache, &key)? {
        read_hash(cache, &entry.integrity)
    } else {
        Err(Error::NotFound)
    }
}

pub fn read_hash(cache: &Path, sri: &Integrity) -> Result<Vec<u8>, Error> {
    Ok(read::read(cache, sri)?)
}

pub fn copy(cache: &Path, key: String, to: &Path) -> Result<u64, Error> {
    if let Some(entry) = index::find(&cache, &key)? {
        copy_hash(cache, &entry.integrity, to)
    } else {
        Err(Error::NotFound)
    }
}

pub fn copy_hash(cache: &Path, sri: &Integrity, to: &Path) -> Result<u64, Error> {
    Ok(read::copy(cache, sri, to)?)
}

pub fn info(cache: &Path, key: String) -> Result<Option<Entry>, Error> {
    index::find(cache, &key)
}

pub fn hash_exists(cache: &Path, sri: &Integrity) -> bool {
    read::has_content(&cache, &sri).is_some()
}
