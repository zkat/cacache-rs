//! Functions for writing to cache.
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use futures::prelude::*;

use anyhow::{Context, Result};
#[cfg(unix)]
use nix::unistd::{Gid, Uid};
use serde_json::Value;
use ssri::{Algorithm, Integrity};

use crate::content::write;
use crate::errors::Error;
use crate::index;

use std::task::{Context as TaskContext, Poll};

/// Writes `data` to the `cache`, indexing it under `key`.
///
/// ## Example
/// ```no_run
/// # use async_std::prelude::*;
/// # use async_std::task;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// # task::block_on(async {
/// #   example().await.unwrap();
/// # });
/// # Ok(())
/// # }
/// #
/// # async fn example() -> Result<()> {
/// cacache::put::data("./my-cache", "my-key", b"hello").await?;
/// # Ok(())
/// # }
/// ```
pub async fn data<P, D, K>(cache: P, key: K, data: D) -> Result<Integrity>
where
    P: AsRef<Path>,
    D: AsRef<[u8]>,
    K: AsRef<str>,
{
    let mut writer = PutOpts::new()
        .algorithm(Algorithm::Sha256)
        .open(cache.as_ref(), key.as_ref())
        .await
        .with_context(|| {
            format!(
                "Failed to open a write handle for key {} for cache at {:?}",
                key.as_ref(),
                cache.as_ref()
            )
        })?;
    writer.write_all(data.as_ref()).await.with_context(|| {
        format!(
            "Failed to write to cache data for key {} for cache at {:?}",
            key.as_ref(),
            cache.as_ref()
        )
    })?;
    writer.commit().await.with_context(|| {
        format!(
            "Failed to write to commit data for key {} for cache at {:?}",
            key.as_ref(),
            cache.as_ref()
        )
    })
}

/// A reference to an open file writing to the cache.
pub struct AsyncPut {
    cache: PathBuf,
    key: String,
    written: usize,
    pub(crate) writer: write::AsyncWriter,
    opts: PutOpts,
}

impl AsyncWrite for AsyncPut {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.writer).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.writer).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.writer).poll_close(cx)
    }
}

impl AsyncPut {
    /// Closes the AsyncPut handle and writes content and index entries. Also
    /// verifies data against `size` and `integrity` options, if provided.
    /// Must be called manually in order to complete the writing process,
    /// otherwise everything will be thrown out.
    pub async fn commit(mut self) -> Result<Integrity> {
        let key = self.key;
        let cache = self.cache;
        let writer_sri = self.writer.close().await.with_context(|| {
            format!(
                "Failed to properly close save file data for key {} in cache at {:?}",
                key, cache
            )
        })?;
        if let Some(sri) = &self.opts.sri {
            if sri.matches(&writer_sri).is_none() {
                return Err(Error::IntegrityError(sri.clone(), writer_sri)).with_context(|| {
                    format!(
                        "Failed to verify data integrity while inserting {} into cache at {:?}",
                        key, cache
                    )
                })?;
            }
        } else {
            self.opts.sri = Some(writer_sri);
        }
        if let Some(size) = self.opts.size {
            if size != self.written {
                return Err(Error::SizeError(size, self.written)).with_context(|| {
                    format!("A size was passed in but the value inserted into {} could not be verified for cache at {:?}", key, cache)
                })?;
            }
        }
        index::insert_async(&cache, &key, self.opts)
            .await
            .with_context(|| {
                format!(
                    "Failed to write index entry for {} in cache at {:?}",
                    key, cache
                )
            })
    }
}

/// Writes `data` to the `cache` synchronously, indexing it under `key`.
///
/// ## Example
/// ```no_run
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// # use std::io::Read;
/// let data = cacache::put::data_sync("./my-cache", "my-key", b"hello")?;
/// # Ok(())
/// # }
/// ```
pub fn data_sync<P, D, K>(cache: P, key: K, data: D) -> Result<Integrity>
where
    P: AsRef<Path>,
    D: AsRef<[u8]>,
    K: AsRef<str>,
{
    let mut writer = PutOpts::new()
        .algorithm(Algorithm::Sha256)
        .open_sync(cache.as_ref(), key.as_ref())
        .with_context(|| {
            format!(
                "Failed to open a write handle for key {} for cache at {:?}",
                key.as_ref(),
                cache.as_ref()
            )
        })?;
    writer.write_all(data.as_ref()).with_context(|| {
        format!(
            "Failed to write to cache data for key {} for cache at {:?}",
            key.as_ref(),
            cache.as_ref()
        )
    })?;
    writer.commit().with_context(|| {
        format!(
            "Failed to write to commit data for key {} for cache at {:?}",
            key.as_ref(),
            cache.as_ref()
        )
    })
}

/// Builder for options and flags for opening a new cache file to write data into.
#[derive(Clone, Default)]
pub struct PutOpts {
    pub(crate) algorithm: Option<Algorithm>,
    pub(crate) sri: Option<Integrity>,
    pub(crate) size: Option<usize>,
    pub(crate) time: Option<u128>,
    pub(crate) metadata: Option<Value>,
    #[cfg(unix)]
    pub(crate) uid: Option<Uid>,
    #[cfg(unix)]
    pub(crate) gid: Option<Gid>,
}

impl PutOpts {
    /// Creates a blank set of cache writing options.
    pub fn new() -> PutOpts {
        Default::default()
    }

    /// Opens the file handle for writing, returning an AsyncPut instance.
    pub async fn open<P, K>(self, cache: P, key: K) -> Result<AsyncPut>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
    {
        Ok(AsyncPut {
            cache: cache.as_ref().to_path_buf(),
            key: String::from(key.as_ref()),
            written: 0,
            writer: write::AsyncWriter::new(
                cache.as_ref(),
                *self.algorithm.as_ref().unwrap_or(&Algorithm::Sha256),
            )
            .await?,
            opts: self,
        })
    }

    /// Opens the file handle for writing synchronously, returning a SyncPut instance.
    pub fn open_sync<P, K>(self, cache: P, key: K) -> Result<SyncPut>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
    {
        Ok(SyncPut {
            cache: cache.as_ref().to_path_buf(),
            key: String::from(key.as_ref()),
            written: 0,
            writer: write::Writer::new(
                cache.as_ref(),
                *self.algorithm.as_ref().unwrap_or(&Algorithm::Sha256),
            )?,
            opts: self,
        })
    }

    /// Configures the algorithm to write data under.
    pub fn algorithm(mut self, algo: Algorithm) -> Self {
        self.algorithm = Some(algo);
        self
    }

    /// Sets the expected size of the data to write. If there's a date size
    /// mismatch, `put.commit()` will return an error.
    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets arbitrary additional metadata to associate with the index entry.
    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the specific time in unix milliseconds to associate with this
    /// entry. This is usually automatically set to the write time, but can be
    /// useful to change for tests and such.
    pub fn time(mut self, time: u128) -> Self {
        self.time = Some(time);
        self
    }

    /// Sets the expected integrity hash of the written data. If there's a
    /// mismatch between this Integrity and the one calculated by the write,
    /// `put.commit()` will error.
    pub fn integrity(mut self, sri: Integrity) -> Self {
        self.sri = Some(sri);
        self
    }

    /// Configures the uid and gid to write data as. Useful when dropping
    /// privileges while in `sudo` mode.
    #[cfg(unix)]
    pub fn chown(mut self, uid: Option<Uid>, gid: Option<Gid>) -> Self {
        self.uid = uid;
        self.gid = gid;
        self
    }
}

/// A reference to an open file writing to the cache.
pub struct SyncPut {
    cache: PathBuf,
    key: String,
    written: usize,
    pub(crate) writer: write::Writer,
    opts: PutOpts,
}

impl Write for SyncPut {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl SyncPut {
    /// Closes the Put handle and writes content and index entries. Also
    /// verifies data against `size` and `integrity` options, if provided.
    /// Must be called manually in order to complete the writing process,
    /// otherwise everything will be thrown out.
    pub fn commit(mut self) -> Result<Integrity> {
        let key = self.key;
        let cache = self.cache;
        let writer_sri = self.writer.close().with_context(|| {
            format!(
                "Failed to properly close save file data for key {} in cache at {:?}",
                key, cache
            )
        })?;
        if let Some(sri) = &self.opts.sri {
            // TODO - ssri should have a .matches method
            if sri.matches(&writer_sri).is_none() {
                return Err(Error::IntegrityError(sri.clone(), writer_sri)).with_context(|| {
                    format!(
                        "Failed to verify data integrity while inserting {} into cache at {:?}",
                        key, cache
                    )
                })?;
            }
        } else {
            self.opts.sri = Some(writer_sri);
        }
        if let Some(size) = self.opts.size {
            if size != self.written {
                return Err(Error::SizeError(size, self.written)).with_context(|| {
                    format!("A size was passed in but the value inserted into {} could not be verified for cache at {:?}", key, cache)
                })?;
            }
        }
        index::insert(&cache, &key, self.opts).with_context(|| {
            format!(
                "Failed to write index entry for {} in cache at {:?}",
                key, cache
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use async_std::task;

    #[test]
    fn round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        task::block_on(async {
            crate::put::data(&dir, "hello", b"hello").await.unwrap();
        });
        let data = task::block_on(async { crate::get::data(&dir, "hello").await.unwrap() });
        assert_eq!(data, b"hello");
    }

    #[test]
    fn round_trip_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        crate::put::data_sync(&dir, "hello", b"hello").unwrap();
        let data = crate::get::data_sync(&dir, "hello").unwrap();
        assert_eq!(data, b"hello");
    }
}
