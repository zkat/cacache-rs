//! Functions for reading from cache.
use std::path::Path;

use ssri::Integrity;

use crate::content::read;
use crate::errors::Error;
use crate::index::{self, Entry};

pub fn read<P: AsRef<Path>, K: AsRef<str>>(cache: P, key: K) -> Result<Vec<u8>, Error> {
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        read_hash(cache, &entry.integrity)
    } else {
        Err(Error::NotFound)
    }
}

pub fn read_hash<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<Vec<u8>, Error> {
    Ok(read::read(cache.as_ref(), sri)?)
}

pub fn copy<P: AsRef<Path>, K: AsRef<str>>(cache: P, key: K, to: P) -> Result<u64, Error> {
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        copy_hash(cache, &entry.integrity, to)
    } else {
        Err(Error::NotFound)
    }
}

pub fn copy_hash<P: AsRef<Path>>(cache: P, sri: &Integrity, to: P) -> Result<u64, Error> {
    Ok(read::copy(cache.as_ref(), sri, to.as_ref())?)
}

pub fn info<P: AsRef<Path>, K: AsRef<str>>(cache: P, key: K) -> Result<Option<Entry>, Error> {
    index::find(cache.as_ref(), key.as_ref())
}

pub fn hash_exists<P: AsRef<Path>>(cache: P, sri: &Integrity) -> bool {
    read::has_content(cache.as_ref(), &sri).is_some()
}
