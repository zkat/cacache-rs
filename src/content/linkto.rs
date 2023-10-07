use ssri::{Algorithm, Integrity, IntegrityOpts};
use std::fs::DirBuilder;
use std::fs::File;
use std::path::{Path, PathBuf};
#[cfg(any(feature = "async-std", feature = "tokio"))]
use std::pin::Pin;
#[cfg(any(feature = "async-std", feature = "tokio"))]
use std::task::{Context, Poll};

#[cfg(any(feature = "async-std", feature = "tokio"))]
use crate::async_lib::AsyncRead;
use crate::content::path;
use crate::errors::{IoErrorExt, Result};

#[cfg(not(any(unix, windows)))]
compile_error!("Symlinking is not supported on this platform.");

fn symlink_file<P, Q>(src: P, dst: Q) -> std::io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(src, dst)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_file;
        symlink_file(src, dst)
    }
}

fn create_symlink(sri: Integrity, cache: &PathBuf, target: &PathBuf) -> Result<Integrity> {
    let cpath = path::content_path(cache.as_ref(), &sri);
    DirBuilder::new()
        .recursive(true)
        // Safe unwrap. cpath always has multiple segments
        .create(cpath.parent().unwrap())
        .with_context(|| {
            format!(
                "Failed to create destination directory for linked cache file, at {}",
                cpath.parent().unwrap().display()
            )
        })?;
    if let Err(e) = symlink_file(target, &cpath) {
        // If symlinking fails because there's *already* a file at the desired
        // destination, that is ok -- all the cache should care about is that
        // there is **some** valid file associated with the computed integrity.
        if !cpath.exists() {
            return Err(e).with_context(|| {
                format!(
                    "Failed to create cache symlink for {} at {}",
                    target.display(),
                    cpath.display()
                )
            });
        }
    }
    Ok(sri)
}

/// A `Read`-like type that calculates the integrity of a file as it is read.
/// When the linker is committed, a symlink is created from the cache to the
/// target file using the integrity computed from the file's contents.
pub struct ToLinker {
    /// The path to the target file that will be symlinked from the cache.
    target: PathBuf,
    /// The path to the root of the cache directory.
    cache: PathBuf,
    /// The file descriptor to the target file.
    fd: File,
    /// The integrity builder for calculating the target file's integrity.
    builder: IntegrityOpts,
}

impl ToLinker {
    pub fn new(cache: &Path, algo: Algorithm, target: &Path) -> Result<Self> {
        let file = File::open(target)
            .with_context(|| format!("Failed to open reader to {}", target.display()))?;
        Ok(Self {
            target: target.to_path_buf(),
            cache: cache.to_path_buf(),
            fd: file,
            builder: IntegrityOpts::new().algorithm(algo),
        })
    }

    /// Add the symlink to the target file from the cache.
    pub fn commit(self) -> Result<Integrity> {
        create_symlink(self.builder.result(), &self.cache, &self.target)
    }
}

impl std::io::Read for ToLinker {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let amt = self.fd.read(buf)?;
        if amt > 0 {
            self.builder.input(&buf[..amt]);
        }
        Ok(amt)
    }
}

/// An `AsyncRead`-like type that calculates the integrity of a file as it is
/// read. When the linker is committed, a symlink is created from the cache to
/// the target file using the integrity computed from the file's contents.
#[cfg(any(feature = "async-std", feature = "tokio"))]
pub struct AsyncToLinker {
    /// The path to the target file that will be symlinked from the cache.
    target: PathBuf,
    /// The path to the root of the cache directory.
    cache: PathBuf,
    /// The async-enabled file descriptor to the target file.
    fd: crate::async_lib::File,
    /// The integrity builder for calculating the target file's integrity.
    builder: IntegrityOpts,
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
impl AsyncRead for AsyncToLinker {
    #[cfg(feature = "async-std")]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let amt = futures::ready!(Pin::new(&mut self.fd).poll_read(cx, buf))?;
        if amt > 0 {
            self.builder.input(&buf[..amt]);
        }
        Poll::Ready(Ok(amt))
    }

    #[cfg(feature = "tokio")]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        let pre_len = buf.filled().len();
        futures::ready!(Pin::new(&mut self.fd).poll_read(cx, buf))?;
        if buf.filled().len() > pre_len {
            self.builder.input(&buf.filled()[pre_len..]);
        }
        Poll::Ready(Ok(()))
    }
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
impl AsyncToLinker {
    pub async fn new(cache: &Path, algo: Algorithm, target: &Path) -> Result<Self> {
        let file = crate::async_lib::File::open(target)
            .await
            .with_context(|| format!("Failed to open reader to {}", target.display()))?;
        Ok(Self {
            target: target.to_path_buf(),
            cache: cache.to_path_buf(),
            fd: file,
            builder: IntegrityOpts::new().algorithm(algo),
        })
    }

    /// Add the symlink to the target file from the cache.
    pub async fn commit(self) -> Result<Integrity> {
        create_symlink(self.builder.result(), &self.cache, &self.target)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use super::*;

    #[cfg(feature = "async-std")]
    use async_attributes::test as async_test;
    #[cfg(feature = "tokio")]
    use tokio::test as async_test;

    #[cfg(feature = "async-std")]
    use futures::io::AsyncReadExt;
    #[cfg(feature = "tokio")]
    use tokio::io::AsyncReadExt;

    fn create_tmpfile(tmp: &tempfile::TempDir, buf: &[u8]) -> PathBuf {
        let dir = tmp.path().to_owned();
        let target = dir.join("target-file");
        std::fs::create_dir_all(&target.parent().unwrap()).unwrap();
        let mut file = File::create(&target).unwrap();
        file.write_all(buf).unwrap();
        file.flush().unwrap();
        target
    }

    #[test]
    fn basic_link() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut linker = ToLinker::new(&dir, Algorithm::Sha256, &target).unwrap();

        // read all of the data from the linker, which will calculate the integrity
        // hash.
        let mut buf = Vec::new();
        linker.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, b"hello world");

        // commit the linker, creating a symlink in the cache and an integrity
        // hash.
        let sri = linker.commit().unwrap();
        assert_eq!(sri.to_string(), Integrity::from(b"hello world").to_string());

        let cpath = path::content_path(&dir, &sri);
        assert!(cpath.exists());
        let metadata = std::fs::symlink_metadata(&cpath).unwrap();
        let file_type = metadata.file_type();
        assert!(file_type.is_symlink());
        assert_eq!(std::fs::read(cpath).unwrap(), b"hello world");
    }

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    #[async_test]
    async fn basic_async_link() {
        let tmp = tempfile::tempdir().unwrap();
        let target = create_tmpfile(&tmp, b"hello world");

        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut linker = AsyncToLinker::new(&dir, Algorithm::Sha256, &target)
            .await
            .unwrap();

        // read all of the data from the linker, which will calculate the integrity
        // hash.
        let mut buf: Vec<u8> = Vec::new();
        AsyncReadExt::read_to_end(&mut linker, &mut buf)
            .await
            .unwrap();
        assert_eq!(buf, b"hello world");

        // commit the linker, creating a symlink in the cache and an integrity
        // hash.
        let sri = linker.commit().await.unwrap();
        assert_eq!(sri.to_string(), Integrity::from(b"hello world").to_string());

        let cpath = path::content_path(&dir, &sri);
        assert!(cpath.exists());
        let metadata = std::fs::symlink_metadata(&cpath).unwrap();
        let file_type = metadata.file_type();
        assert!(file_type.is_symlink());
        assert_eq!(std::fs::read(cpath).unwrap(), b"hello world");
    }
}
