use crate::async_lib::AsyncRead;
use crate::content::symlink;
use crate::errors::{Error, IoErrorExt, Result};
use crate::{index, WriteOpts};
use ssri::{Algorithm, Integrity};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};

#[cfg(feature = "async-std")]
use futures::io::AsyncReadExt;
#[cfg(feature = "tokio")]
use tokio::io::AsyncReadExt;

const BUF_SIZE: usize = 16 * 1024;
const PROBE_SIZE: usize = 8;

/// Asynchronously adds `target` to the `cache` with a symlink, indexing it
/// under `key`.
///
/// ## Example
/// ```no_run
/// use async_attributes;
/// use std::path::Path;
///
/// #[async_attributes::main]
/// async fn main() -> cacache::Result<()> {
///     cacache::link("./my-cache", "my-key", "../my-other-files/my-file.tgz").await?;
///     Ok(())
/// }
/// ```
pub async fn link<P, K, T>(cache: P, key: K, target: T) -> Result<Integrity>
where
    P: AsRef<Path>,
    K: AsRef<str>,
    T: AsRef<Path>,
{
    Linker::open(cache, key, target).await?.commit().await
}

/// Asynchrounously adds `target` to the `cache` with a symlink, skipping
/// associating an index key with it.
///
/// ## Example
/// ```no_run
/// use async_attributes;
/// use std::path::Path;
///
/// #[async_attributes::main]
/// async fn main() -> cacache::Result<()> {
///     cacache::link_hash("./my-cache", "../my-other-files/my-file.tgz").await?;
///     Ok(())
/// }
/// ```
pub async fn link_hash<P, T>(cache: P, target: T) -> Result<Integrity>
where
    P: AsRef<Path>,
    T: AsRef<Path>,
{
    Linker::open_hash(cache, target).await?.commit().await
}

/// Synchronously adds `target` to the `cache` with a symlink, indexing it under
/// `key`.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
/// use std::path::Path;
///
/// fn main() -> cacache::Result<()> {
///     cacache::link_sync("./my-cache", "my-key", "../my-other-files/my-file.tgz")?;
///     Ok(())
/// }
/// ```
pub fn link_sync<P, K, T>(cache: P, key: K, target: T) -> Result<Integrity>
where
    P: AsRef<Path>,
    K: AsRef<str>,
    T: AsRef<Path>,
{
    SyncLinker::open(cache, key, target)?.commit()
}

/// Synchronously adds `target` to the `cache` with a symlink, skipping
/// associating an index key with it.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
/// use std::path::Path;
///
/// fn main() -> cacache::Result<()> {
///     cacache::link_hash_sync("./my-cache", "../foo/bar.tgz")?;
///     Ok(())
/// }
/// ```
pub fn link_hash_sync<P, T>(cache: P, target: T) -> Result<Integrity>
where
    P: AsRef<Path>,
    T: AsRef<Path>,
{
    SyncLinker::open_hash(cache, target)?.commit()
}

/// Extend the `WriteOpts` struct with factories for creating `Linker` and
/// `SyncLinker` instances.
impl WriteOpts {
    /// Opens the target file handle for reading, returning a Linker instance.
    pub async fn link<P, K, T>(self, cache: P, key: K, target: T) -> Result<Linker>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
        T: AsRef<Path>,
    {
        async fn inner(opts: WriteOpts, cache: &Path, key: &str, target: &Path) -> Result<Linker> {
            Ok(Linker {
                cache: cache.to_path_buf(),
                key: Some(String::from(key)),
                read: 0,
                linker: symlink::AsyncLinker::new(
                    cache,
                    opts.algorithm.unwrap_or(Algorithm::Sha256),
                    target,
                )
                .await?,
                opts,
            })
        }
        inner(self, cache.as_ref(), key.as_ref(), target.as_ref()).await
    }

    /// Opens the target file handle for reading, without a key, returning a
    /// Linker instance.
    pub async fn link_hash<P, T>(self, cache: P, target: T) -> Result<Linker>
    where
        P: AsRef<Path>,
        T: AsRef<Path>,
    {
        async fn inner(opts: WriteOpts, cache: &Path, target: &Path) -> Result<Linker> {
            Ok(Linker {
                cache: cache.to_path_buf(),
                key: None,
                read: 0,
                linker: symlink::AsyncLinker::new(
                    cache,
                    opts.algorithm.unwrap_or(Algorithm::Sha256),
                    target,
                )
                .await?,
                opts,
            })
        }
        inner(self, cache.as_ref(), target.as_ref()).await
    }

    /// Opens the target file handle for reading synchronously, returning a
    /// SyncLinker instance.
    pub fn link_sync<P, K, T>(self, cache: P, key: K, target: T) -> Result<SyncLinker>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
        T: AsRef<Path>,
    {
        fn inner(opts: WriteOpts, cache: &Path, key: &str, target: &Path) -> Result<SyncLinker> {
            Ok(SyncLinker {
                cache: cache.to_path_buf(),
                key: Some(String::from(key)),
                read: 0,
                linker: symlink::Linker::new(
                    cache,
                    opts.algorithm.unwrap_or(Algorithm::Sha256),
                    target,
                )?,
                opts,
            })
        }
        inner(self, cache.as_ref(), key.as_ref(), target.as_ref())
    }

    /// Opens the target file handle for reading synchronously, without a key,
    /// returning a SyncLinker instance.
    pub fn link_hash_sync<P, T>(self, cache: P, target: T) -> Result<SyncLinker>
    where
        P: AsRef<Path>,
        T: AsRef<Path>,
    {
        fn inner(opts: WriteOpts, cache: &Path, target: &Path) -> Result<SyncLinker> {
            Ok(SyncLinker {
                cache: cache.to_path_buf(),
                key: None,
                read: 0,
                linker: symlink::Linker::new(
                    cache,
                    opts.algorithm.unwrap_or(Algorithm::Sha256),
                    target,
                )?,
                opts,
            })
        }
        inner(self, cache.as_ref(), target.as_ref())
    }
}

/// A file handle for asynchronously reading data from a file to be added to the
/// cache via a symlink.
///
/// Make sure to call `.commit()` when done reading to actually add the file to
/// the cache.
pub struct Linker {
    cache: PathBuf,
    key: Option<String>,
    read: usize,
    pub(crate) linker: symlink::AsyncLinker,
    opts: WriteOpts,
}

impl AsyncRead for Linker {
    #[cfg(feature = "async-std")]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let amt = futures::ready!(Pin::new(&mut self.linker).poll_read(cx, buf))?;
        self.read += amt;
        Poll::Ready(Ok(amt))
    }

    #[cfg(feature = "tokio")]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        let pre_len = buf.filled().len();
        futures::ready!(Pin::new(&mut self.linker).poll_read(cx, buf))?;
        self.read += buf.filled().len() - pre_len;
        Poll::Ready(Ok(()))
    }
}

impl Linker {
    /// Creates a new asynchronous readable file handle into the cache.
    pub async fn open<P, K, T>(cache: P, key: K, target: T) -> Result<Self>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
        T: AsRef<Path>,
    {
        async fn inner(cache: &Path, key: &str, target: &Path) -> Result<Linker> {
            let size = target
                .metadata()
                .with_context(|| format!("Failed to get metadata of {}", target.display()))?
                .len() as usize;
            WriteOpts::new()
                .algorithm(Algorithm::Sha256)
                .size(size)
                .link(cache, key, target)
                .await
        }
        inner(cache.as_ref(), key.as_ref(), target.as_ref()).await
    }

    /// Creates a new asynchronous readable file handle into the cache.
    pub async fn open_hash<P, T>(cache: P, target: T) -> Result<Self>
    where
        P: AsRef<Path>,
        T: AsRef<Path>,
    {
        async fn inner(cache: &Path, target: &Path) -> Result<Linker> {
            let size = target
                .metadata()
                .with_context(|| format!("Failed to get metadata of {}", target.display()))?
                .len() as usize;
            WriteOpts::new()
                .algorithm(Algorithm::Sha256)
                .size(size)
                .link_hash(cache, target)
                .await
        }
        inner(cache.as_ref(), target.as_ref()).await
    }

    /// Consumes the rest of the file handle, creates a symlink into the cache,
    /// and creates index entries for the linked file. Also verifies data
    /// against `size` and `integrity` options, if provided. Must be called
    /// manually in order to complete the writing process, otherwise everything
    /// will be thrown out.
    pub async fn commit(mut self) -> Result<Integrity> {
        self.consume().await?;
        let linker_sri = self.linker.commit().await?;
        if let Some(sri) = &self.opts.sri {
            if sri.matches(&linker_sri).is_none() {
                return Err(ssri::Error::IntegrityCheckError(sri.clone(), linker_sri).into());
            }
        } else {
            self.opts.sri = Some(linker_sri.clone());
        }
        if let Some(size) = self.opts.size {
            if size != self.read {
                return Err(Error::SizeMismatch(size, self.read));
            }
        }
        if let Some(key) = self.key {
            index::insert(&self.cache, &key, self.opts)
        } else {
            Ok(linker_sri)
        }
    }

    // "Consume" the remainder of the reader, so that the integrity is properly
    // calculated.
    async fn consume(&mut self) -> Result<()> {
        // Do a small 'test' read to avoid allocating a larger buffer if it
        // isn't necessary.
        let mut probe = [0; PROBE_SIZE];
        if self.context_read(&mut probe).await? > 0 {
            // Make sure all the bytes are read so that the integrity is
            // properly calculated.
            let mut buf = [0; BUF_SIZE];
            while self.context_read(&mut buf).await? > 0 {}
        }
        Ok(())
    }

    async fn context_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        AsyncReadExt::read(self, buf).await.with_context(|| {
            "Failed to read target file contents while calculating integrity".into()
        })
    }
}

/// A file handle for synchronously reading data from a file to be added to the
/// cache via a symlink.
///
/// Make sure to call `.commit()` when done reading to actually add the file
/// to the cache.
pub struct SyncLinker {
    cache: PathBuf,
    key: Option<String>,
    read: usize,
    pub(crate) linker: symlink::Linker,
    opts: WriteOpts,
}

impl std::io::Read for SyncLinker {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let amt = self.linker.read(buf)?;
        self.read += amt;
        Ok(amt)
    }
}

impl SyncLinker {
    /// Creates a new readable file handle to a file that will be linked into
    /// the cache, indexed at the provided key, on commit.
    ///
    /// It is not necessary to read any of the file before calling `.commit()`.
    ///
    /// ## Example
    /// ```no_run
    /// use std::io::prelude::*;
    ///
    /// fn main() -> cacache::Result<()> {
    ///     let path = "../my-other-files/my-file.tgz";
    ///     let mut fd = cacache::SyncLinker::open("./my-cache", "my-key", path)?;
    ///     let mut str = String::new();
    ///     fd.read_to_string(&mut str).expect("Failed to read to string");
    ///     // The file is not linked into the cache until you commit it.
    ///     fd.commit()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn open<P, K, T>(cache: P, key: K, target: T) -> Result<Self>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
        T: AsRef<Path>,
    {
        fn inner(cache: &Path, key: &str, target: &Path) -> Result<SyncLinker> {
            let size = target
                .metadata()
                .with_context(|| format!("Failed to get metadata of {}", target.display()))?
                .len() as usize;
            WriteOpts::new()
                .algorithm(Algorithm::Sha256)
                .size(size)
                .link_sync(cache, key, target)
        }
        inner(cache.as_ref(), key.as_ref(), target.as_ref())
    }

    /// Creates a new readable file handle to a file that will be linked into
    /// the cache, without an indexe key, on commit.
    ///
    /// It is not necessary to read any of the file before calling `.commit()`.
    ///
    /// ## Example
    /// ```no_run
    /// use std::io::prelude::*;
    ///
    /// fn main() -> cacache::Result<()> {
    ///     let path = "../my-other-files/my-file.tgz";
    ///     let mut fd = cacache::SyncLinker::open_hash("./my-cache", path)?;
    ///     let mut str = String::new();
    ///     fd.read_to_string(&mut str).expect("Failed to read to string");
    ///     // The file is not linked into the cache until you commit it.
    ///     fd.commit()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn open_hash<P, T>(cache: P, target: T) -> Result<Self>
    where
        P: AsRef<Path>,
        T: AsRef<Path>,
    {
        fn inner(cache: &Path, target: &Path) -> Result<SyncLinker> {
            let size = target
                .metadata()
                .with_context(|| format!("Failed to get metadata of {}", target.display()))?
                .len() as usize;
            WriteOpts::new()
                .algorithm(Algorithm::Sha256)
                .size(size)
                .link_hash_sync(cache, target)
        }
        inner(cache.as_ref(), target.as_ref())
    }

    /// Consumes the rest of the file handle, creates a symlink into the cache,
    /// and creates index entries for the linked file. Also verifies data
    /// against `size` and `integrity` options, if provided. Must be called
    /// manually in order to complete the writing process, otherwise everything
    /// will be thrown out.
    pub fn commit(mut self) -> Result<Integrity> {
        self.consume()?;
        let cache = self.cache;
        let linker_sri = self.linker.commit()?;
        if let Some(sri) = &self.opts.sri {
            if sri.matches(&linker_sri).is_none() {
                return Err(ssri::Error::IntegrityCheckError(sri.clone(), linker_sri).into());
            }
        } else {
            self.opts.sri = Some(linker_sri.clone());
        }
        if let Some(size) = self.opts.size {
            if size != self.read {
                return Err(Error::SizeMismatch(size, self.read));
            }
        }
        if let Some(key) = self.key {
            index::insert(&cache, &key, self.opts)
        } else {
            Ok(linker_sri)
        }
    }

    fn consume(&mut self) -> Result<()> {
        // Do a small 'test' read to avoid allocating a larger buffer if it
        // isn't necessary.
        let mut probe = [0; PROBE_SIZE];
        if self.context_read(&mut probe)? > 0 {
            // Make sure all the bytes are read so that the integrity is
            // properly calculated.
            let mut buf = [0; BUF_SIZE];
            while self.context_read(&mut buf)? > 0 {}
        }
        Ok(())
    }

    fn context_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.read(buf).with_context(|| {
            "Failed to read target file contents while calculating integrity".into()
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use super::*;

    #[cfg(feature = "async-std")]
    use async_attributes::test as async_test;
    #[cfg(feature = "tokio")]
    use tokio::test as async_test;

    fn create_tmpfile(tmp: &tempfile::TempDir, buf: &[u8]) -> PathBuf {
        let dir = tmp.path().to_owned();
        let target = dir.join("target-file");
        std::fs::create_dir_all(target.parent().unwrap().clone()).unwrap();
        let mut file = File::create(target.clone()).unwrap();
        file.write_all(buf).unwrap();
        file.flush().unwrap();
        target
    }

    #[async_test]
    async fn test_link() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        crate::link(&dir, "my-key", target).await.unwrap();

        let buf = crate::read(&dir, "my-key").await.unwrap();
        assert_eq!(buf, b"hello world");
    }

    #[async_test]
    async fn test_link_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::link_hash(&dir, target).await.unwrap();

        let buf = crate::read_hash(&dir, &sri).await.unwrap();
        assert_eq!(buf, b"hello world");
    }

    #[test]
    fn test_link_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        crate::link_sync(&dir, "my-key", target).unwrap();

        let buf = crate::read_sync(&dir, "my-key").unwrap();
        assert_eq!(buf, b"hello world");
    }

    #[test]
    fn test_link_hash_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::link_hash_sync(&dir, target).unwrap();

        let buf = crate::read_hash_sync(&dir, &sri).unwrap();
        assert_eq!(buf, b"hello world");
    }

    #[async_test]
    async fn test_open() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut handle = crate::Linker::open(&dir, "my-key", target).await.unwrap();

        let mut buf = Vec::new();
        handle.read_to_end(&mut buf).await.unwrap();
        handle.commit().await.unwrap();
        assert_eq!(buf, b"hello world");

        let buf = crate::read_sync(&dir, "my-key").unwrap();
        assert_eq!(buf, b"hello world");
    }

    #[async_test]
    async fn test_open_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut handle = crate::Linker::open_hash(&dir, target).await.unwrap();

        let mut buf = Vec::new();
        handle.read_to_end(&mut buf).await.unwrap();
        let sri = handle.commit().await.unwrap();
        assert_eq!(buf, b"hello world");

        let buf = crate::read_hash_sync(&dir, &sri).unwrap();
        assert_eq!(buf, b"hello world");
    }

    #[test]
    fn test_open_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut handle = crate::SyncLinker::open(&dir, "my-key", target).unwrap();

        let mut buf = Vec::new();
        handle.read_to_end(&mut buf).unwrap();
        handle.commit().unwrap();
        assert_eq!(buf, b"hello world");

        let buf = crate::read_sync(&dir, "my-key").unwrap();
        assert_eq!(buf, b"hello world");
    }

    #[test]
    fn test_open_hash_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut handle = crate::SyncLinker::open_hash(&dir, target).unwrap();

        let mut buf = Vec::new();
        handle.read_to_end(&mut buf).unwrap();
        let sri = handle.commit().unwrap();
        assert_eq!(buf, b"hello world");

        let buf = crate::read_hash_sync(&dir, &sri).unwrap();
        assert_eq!(buf, b"hello world");
    }
}
