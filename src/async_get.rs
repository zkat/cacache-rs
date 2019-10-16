//! Functions for reading asynchronously from cache.
//!
//! Asynchronous operations are able to trade off some linear performance in
//! exchange for potentially much higher performance on heavily-concurrent
//! loads.
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::prelude::*;
use ssri::{Algorithm, Integrity};

use crate::content::read::{self, AsyncReader};
use crate::errors::Error;
use crate::index::{self, Entry};

/// File handle for asynchronously reading from a content entry.
///
/// Make sure to call `.check()` when done reading to verify that the
/// extracted data passes integrity verification.
pub struct AsyncGet {
    reader: AsyncReader,
}

impl AsyncRead for AsyncGet {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

impl AsyncGet {
    /// Checks that data read from disk passes integrity checks. Returns the
    /// algorithm that was used verified the data. Should be called only after
    /// all data has been read from disk.
    pub fn check(self) -> Result<Algorithm, Error> {
        self.reader.check()
    }
}

/// Opens a new file handle into the cache, looking it up in the index using
/// `key`.
pub async fn open<P, K>(cache: P, key: K) -> Result<AsyncGet, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    if let Some(entry) = index::find_async(cache.as_ref(), key.as_ref()).await? {
        open_hash(cache, entry.integrity).await
    } else {
        Err(Error::NotFound)
    }
}

/// Opens a new file handle into the cache, based on its integrity address.
pub async fn open_hash<P>(cache: P, sri: Integrity) -> Result<AsyncGet, Error>
where
    P: AsRef<Path>,
{
    Ok(AsyncGet {
        reader: read::open_async(cache.as_ref(), sri).await?,
    })
}

/// Reads the entire contents of a cache file into a bytes vector, looking the
/// data up by key.
pub async fn data<P, K>(cache: P, key: K) -> Result<Vec<u8>, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    if let Some(entry) = index::find_async(cache.as_ref(), key.as_ref()).await? {
        data_hash(cache, &entry.integrity).await
    } else {
        Err(Error::NotFound)
    }
}

/// Reads the entire contents of a cache file into a bytes vector, looking the
/// data up by its content address.
pub async fn data_hash<P>(cache: P, sri: &Integrity) -> Result<Vec<u8>, Error>
where
    P: AsRef<Path>,
{
    Ok(read::read_async(cache.as_ref(), sri).await?)
}

/// Copies a cache entry by key to a specified location.
pub async fn copy<P, K, Q>(cache: P, key: K, to: Q) -> Result<u64, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
    Q: AsRef<Path>,
{
    if let Some(entry) = index::find_async(cache.as_ref(), key.as_ref()).await? {
        copy_hash(cache, &entry.integrity, to).await
    } else {
        Err(Error::NotFound)
    }
}

/// Copies a cache entry by integrity address to a specified location.
pub async fn copy_hash<P, Q>(cache: P, sri: &Integrity, to: Q) -> Result<u64, Error>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    read::copy_async(cache.as_ref(), sri, to.as_ref()).await
}

/// Gets entry information and metadata for a certain key.
pub async fn info<P, K>(cache: P, key: K) -> Result<Option<Entry>, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    index::find_async(cache.as_ref(), key.as_ref()).await
}

/// Returns true if the given hash exists in the cache.
pub async fn hash_exists<P: AsRef<Path>>(cache: P, sri: &Integrity) -> bool {
    read::has_content_async(cache.as_ref(), &sri)
        .await
        .is_some()
}
