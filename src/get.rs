//! Functions for reading from cache.
use std::path::Path;

use ssri::{Algorithm, Integrity};

use crate::content::read::{self, Reader};
use crate::errors::Error;
use crate::index::{self, Entry};

/// File handle for reading from a content entry.
///
/// Make sure to call `get.check()` when done reading
/// to verify that the extracted data passes integrity
/// verification.
pub struct Get {
    reader: Reader,
}

impl std::io::Read for Get {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Get {
    /// Checks that data read from disk passes integrity checks. Returns the
    /// algorithm that was used verified the data. Should be called only after
    /// all data has been read from disk.
    pub fn check(self) -> Result<Algorithm, Error> {
        self.reader.check()
    }
}

/// Opens a new file handle into the cache, looking it up in the index using
/// `key`.
pub fn open<P, K>(cache: P, key: K) -> Result<Get, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        open_hash(cache, entry.integrity)
    } else {
        Err(Error::NotFound)
    }
}

/// Opens a new file handle into the cache, based on its integrity address.
pub fn open_hash<P>(cache: P, sri: Integrity) -> Result<Get, Error>
where
    P: AsRef<Path>,
{
    Ok(Get {
        reader: read::open(cache.as_ref(), sri)?,
    })
}

/// Reads the entire contents of a cache file into a bytes vector, looking the
/// data up by key.
pub fn read<P, K>(cache: P, key: K) -> Result<Vec<u8>, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        read_hash(cache, &entry.integrity)
    } else {
        Err(Error::NotFound)
    }
}

/// Reads the entire contents of a cache file into a bytes vector, looking the
/// data up by its content address.
pub fn read_hash<P>(cache: P, sri: &Integrity) -> Result<Vec<u8>, Error>
where
    P: AsRef<Path>,
{
    Ok(read::read(cache.as_ref(), sri)?)
}

/// Copies a cache entry by key to a specified location.
pub fn copy<P, K, Q>(cache: P, key: K, to: Q) -> Result<u64, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
    Q: AsRef<Path>,
{
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        copy_hash(cache, &entry.integrity, to)
    } else {
        Err(Error::NotFound)
    }
}

/// Copies a cache entry by integrity address to a specified location.
pub fn copy_hash<P, Q>(cache: P, sri: &Integrity, to: Q) -> Result<u64, Error>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    read::copy(cache.as_ref(), sri, to.as_ref())
}

/// Gets entry information and metadata for a certain key.
pub fn info<P, K>(cache: P, key: K) -> Result<Option<Entry>, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    index::find(cache.as_ref(), key.as_ref())
}

/// Returns true if the given hash exists in the cache.
pub fn hash_exists<P: AsRef<Path>>(cache: P, sri: &Integrity) -> bool {
    read::has_content(cache.as_ref(), &sri).is_some()
}
