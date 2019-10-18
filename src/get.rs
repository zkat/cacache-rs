//! Functions for reading from cache.
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::prelude::*;

use ssri::{Algorithm, Integrity};

use crate::content::read::{self, AsyncReader, Reader};
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
///
/// ## Example
/// ```no_run
/// # use async_std::prelude::*;
/// # use async_std::task;
/// # fn main() -> Result<(), cacache::Error> {
/// # task::block_on(async {
/// #   example().await.unwrap();
/// # });
/// # Ok(())
/// # }
/// #
/// # async fn example() -> Result<(), cacache::Error> {
/// let mut handle = cacache::get::open("./my-cache", "my-key").await?;
/// let mut str = String::new();
/// handle.read_to_string(&mut str).await?;
/// // Remember to check that the data you got was correct!
/// handle.check()?;
/// # Ok(())
/// # }
/// ```
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
///
/// ## Example
/// ```no_run
/// # use async_std::prelude::*;
/// # use async_std::task;
/// # fn main() -> Result<(), cacache::Error> {
/// # task::block_on(async {
/// #   example().await.unwrap();
/// # });
/// # Ok(())
/// # }
/// #
/// # async fn example() -> Result<(), cacache::Error> {
/// let sri = cacache::put::data("./my-cache", "key", b"hello world").await?;
/// let mut handle = cacache::get::open_hash("./my-cache", sri).await?;
/// let mut str = String::new();
/// handle.read_to_string(&mut str).await?;
/// // Remember to check that the data you got was correct!
/// handle.check()?;
/// # Ok(())
/// # }
/// ```
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
///
/// ## Example
/// ```no_run
/// # use async_std::prelude::*;
/// # use async_std::task;
/// # fn main() -> Result<(), cacache::Error> {
/// # task::block_on(async {
/// #   example().await.unwrap();
/// # });
/// # Ok(())
/// # }
/// #
/// # async fn example() -> Result<(), cacache::Error> {
/// let data = cacache::get::data("./my-cache", "my-key").await?;
/// # Ok(())
/// # }
/// ```
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
///
/// ## Example
/// ```no_run
/// # use async_std::prelude::*;
/// # use async_std::task;
/// # fn main() -> Result<(), cacache::Error> {
/// # task::block_on(async {
/// #   example().await.unwrap();
/// # });
/// # Ok(())
/// # }
/// #
/// # async fn example() -> Result<(), cacache::Error> {
/// let sri = cacache::put::data("./my-cache", "my-key", b"hello").await?;
/// let data = cacache::get::data_hash("./my-cache", &sri).await?;
/// # Ok(())
/// # }
/// ```
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

/// Opens a new synchronous file handle into the cache, looking it up in the
/// index using `key`.
///
/// ## Example
/// ```no_run
/// # fn main() -> Result<(), cacache::Error> {
/// # use std::io::Read;
/// let mut handle = cacache::get::open_sync("./my-cache", "my-key")?;
/// let mut str = String::new();
/// handle.read_to_string(&mut str)?;
/// // Remember to check that the data you got was correct!
/// handle.check()?;
/// # Ok(())
/// # }
/// ```
pub fn open_sync<P, K>(cache: P, key: K) -> Result<Get, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        open_hash_sync(cache, entry.integrity)
    } else {
        Err(Error::NotFound)
    }
}

/// Opens a new synchronous file handle into the cache, based on its integrity address.
///
/// ## Example
/// ```no_run
/// # fn main() -> Result<(), cacache::Error> {
/// # use std::io::Read;
/// let sri = cacache::put::data_sync("./my-cache", "key", b"hello world")?;
/// let mut handle = cacache::get::open_hash_sync("./my-cache", sri)?;
/// let mut str = String::new();
/// handle.read_to_string(&mut str)?;
/// // Remember to check that the data you got was correct!
/// handle.check()?;
/// # Ok(())
/// # }
/// ```
pub fn open_hash_sync<P>(cache: P, sri: Integrity) -> Result<Get, Error>
where
    P: AsRef<Path>,
{
    Ok(Get {
        reader: read::open(cache.as_ref(), sri)?,
    })
}

/// Reads the entire contents of a cache file synchronously into a bytes
/// vector, looking the data up by key.
///
/// ## Example
/// ```no_run
/// # fn main() -> Result<(), cacache::Error> {
/// # use std::io::Read;
/// let data = cacache::get::data_sync("./my-cache", "my-key")?;
/// # Ok(())
/// # }
/// ```
pub fn data_sync<P, K>(cache: P, key: K) -> Result<Vec<u8>, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        data_hash_sync(cache, &entry.integrity)
    } else {
        Err(Error::NotFound)
    }
}

/// Reads the entire contents of a cache file synchronously into a bytes
/// vector, looking the data up by its content address.
///
/// ## Example
/// ```no_run
/// # fn main() -> Result<(), cacache::Error> {
/// # use std::io::Read;
/// let sri = cacache::put::data_sync("./my-cache", "my-key", b"hello")?;
/// let data = cacache::get::data_hash_sync("./my-cache", &sri)?;
/// # Ok(())
/// # }
/// ```
pub fn data_hash_sync<P>(cache: P, sri: &Integrity) -> Result<Vec<u8>, Error>
where
    P: AsRef<Path>,
{
    Ok(read::read(cache.as_ref(), sri)?)
}

/// Copies a cache entry by key to a specified location.
pub fn copy_sync<P, K, Q>(cache: P, key: K, to: Q) -> Result<u64, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
    Q: AsRef<Path>,
{
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        copy_hash_sync(cache, &entry.integrity, to)
    } else {
        Err(Error::NotFound)
    }
}

/// Copies a cache entry by integrity address to a specified location.
pub fn copy_hash_sync<P, Q>(cache: P, sri: &Integrity, to: Q) -> Result<u64, Error>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    read::copy(cache.as_ref(), sri, to.as_ref())
}

/// Gets entry information and metadata for a certain key.
pub fn info_sync<P, K>(cache: P, key: K) -> Result<Option<Entry>, Error>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    index::find(cache.as_ref(), key.as_ref())
}

/// Returns true if the given hash exists in the cache.
pub fn hash_exists_sync<P: AsRef<Path>>(cache: P, sri: &Integrity) -> bool {
    read::has_content(cache.as_ref(), &sri).is_some()
}

#[cfg(test)]
mod tests {
    use async_std::prelude::*;
    use async_std::task;
    use tempfile;

    #[test]
    fn test_open() {
        task::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            crate::put::data(&dir, "my-key", b"hello world")
                .await
                .unwrap();

            let mut handle = crate::get::open(&dir, "my-key").await.unwrap();
            let mut str = String::new();
            handle.read_to_string(&mut str).await.unwrap();
            handle.check().unwrap();
            assert_eq!(str, String::from("hello world"));
        });
    }

    #[test]
    fn test_open_hash() {
        task::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            let sri = crate::put::data(&dir, "my-key", b"hello world")
                .await
                .unwrap();

            let mut handle = crate::get::open_hash(&dir, sri).await.unwrap();
            let mut str = String::new();
            handle.read_to_string(&mut str).await.unwrap();
            handle.check().unwrap();
            assert_eq!(str, String::from("hello world"));
        });
    }

    #[test]
    fn test_open_sync() {
        use std::io::prelude::*;
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        crate::put::data_sync(&dir, "my-key", b"hello world").unwrap();

        let mut handle = crate::get::open_sync(&dir, "my-key").unwrap();
        let mut str = String::new();
        handle.read_to_string(&mut str).unwrap();
        handle.check().unwrap();
        assert_eq!(str, String::from("hello world"));
    }

    #[test]
    fn test_open_hash_sync() {
        use std::io::prelude::*;
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::put::data_sync(&dir, "my-key", b"hello world").unwrap();

        let mut handle = crate::get::open_hash_sync(&dir, sri).unwrap();
        let mut str = String::new();
        handle.read_to_string(&mut str).unwrap();
        handle.check().unwrap();
        assert_eq!(str, String::from("hello world"));
    }

    #[test]
    fn test_data() {
        task::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            crate::put::data(&dir, "my-key", b"hello world")
                .await
                .unwrap();

            let data = crate::get::data(&dir, "my-key").await.unwrap();
            assert_eq!(data, b"hello world");
        });
    }

    #[test]
    fn test_data_hash() {
        task::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            let sri = crate::put::data(&dir, "my-key", b"hello world")
                .await
                .unwrap();

            let data = crate::get::data_hash(&dir, &sri).await.unwrap();
            assert_eq!(data, b"hello world");
        });
    }

    #[test]
    fn test_data_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        crate::put::data_sync(&dir, "my-key", b"hello world").unwrap();

        let data = crate::get::data_sync(&dir, "my-key").unwrap();
        assert_eq!(data, b"hello world");
    }

    #[test]
    fn test_data_hash_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::put::data_sync(&dir, "my-key", b"hello world").unwrap();

        let data = crate::get::data_hash_sync(&dir, &sri).unwrap();
        assert_eq!(data, b"hello world");
    }
}
