use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
#[cfg(any(feature = "async-std", feature = "tokio"))]
use std::pin::Pin;
#[cfg(any(feature = "async-std", feature = "tokio"))]
use std::task::{Context, Poll};

#[cfg(any(feature = "async-std", feature = "tokio"))]
use crate::async_lib::AsyncReadExt;

use ssri::{Algorithm, Integrity, IntegrityChecker};

#[cfg(any(feature = "async-std", feature = "tokio"))]
use crate::async_lib::AsyncRead;
use crate::content::path;
use crate::errors::{IoErrorExt, Result};

pub struct Reader {
    fd: File,
    checker: IntegrityChecker,
}

impl std::io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let amt = self.fd.read(buf)?;
        self.checker.input(&buf[..amt]);
        Ok(amt)
    }
}

impl Reader {
    pub fn check(self) -> Result<Algorithm> {
        Ok(self.checker.result()?)
    }
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub struct AsyncReader {
    fd: crate::async_lib::File,
    checker: IntegrityChecker,
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
impl AsyncRead for AsyncReader {
    #[cfg(feature = "async-std")]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let amt = futures::ready!(Pin::new(&mut self.fd).poll_read(cx, buf))?;
        self.checker.input(&buf[..amt]);
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
        let post_len = buf.filled().len();
        if post_len - pre_len == 0 {
            return Poll::Ready(Ok(()));
        }
        self.checker.input(&buf.filled()[pre_len..]);
        Poll::Ready(Ok(()))
    }
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
impl AsyncReader {
    pub fn check(self) -> Result<Algorithm> {
        Ok(self.checker.result()?)
    }
}

pub fn open(cache: &Path, sri: Integrity) -> Result<Reader> {
    let cpath = path::content_path(cache, &sri);
    Ok(Reader {
        fd: File::open(cpath).with_context(|| {
            format!(
                "Failed to open reader to {}",
                path::content_path(cache, &sri).display()
            )
        })?,
        checker: IntegrityChecker::new(sri),
    })
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn open_async(cache: &Path, sri: Integrity) -> Result<AsyncReader> {
    let cpath = path::content_path(cache, &sri);
    Ok(AsyncReader {
        fd: crate::async_lib::File::open(cpath).await.with_context(|| {
            format!(
                "Failed to open reader to {}",
                path::content_path(cache, &sri).display()
            )
        })?,
        checker: IntegrityChecker::new(sri),
    })
}

pub fn read(cache: &Path, sri: &Integrity) -> Result<Vec<u8>> {
    let cpath = path::content_path(cache, sri);
    let ret = fs::read(cpath).with_context(|| {
        format!(
            "Failed to read contents for file at {}",
            path::content_path(cache, sri).display()
        )
    })?;
    sri.check(&ret)?;
    Ok(ret)
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn read_async<'a>(cache: &'a Path, sri: &'a Integrity) -> Result<Vec<u8>> {
    let cpath = path::content_path(cache, sri);
    let ret = crate::async_lib::read(&cpath).await.with_context(|| {
        format!(
            "Failed to read contents for file at {}",
            path::content_path(cache, sri).display()
        )
    })?;
    sri.check(&ret)?;
    Ok(ret)
}

pub fn reflink_unchecked(cache: &Path, sri: &Integrity, to: &Path) -> Result<()> {
    let cpath = path::content_path(cache, sri);
    reflink_copy::reflink(cpath, to).with_context(|| {
        format!(
            "Failed to reflink cache contents from {} to {}",
            path::content_path(cache, sri).display(),
            to.display()
        )
    })?;
    Ok(())
}

pub fn reflink(cache: &Path, sri: &Integrity, to: &Path) -> Result<()> {
    let mut reader = open(cache, sri.clone())?;
    let mut buf: [u8; 1024] = [0; 1024];
    loop {
        let read = reader.read(&mut buf).with_context(|| {
            format!(
                "Failed to read cache contents while verifying integrity for {}",
                path::content_path(cache, sri).display()
            )
        })?;
        if read == 0 {
            break;
        }
    }
    reader.check()?;
    reflink_unchecked(cache, sri, to)
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn reflink_async(cache: &Path, sri: &Integrity, to: &Path) -> Result<()> {
    let mut reader = open_async(cache, sri.clone()).await?;
    let mut buf = [0u8; 1024 * 8];
    loop {
        let read = AsyncReadExt::read(&mut reader, &mut buf)
            .await
            .with_context(|| {
                format!(
                    "Failed to read cache contents while verifying integrity for {}",
                    path::content_path(cache, sri).display()
                )
            })?;
        if read == 0 {
            break;
        }
    }
    reader.check()?;
    reflink_unchecked(cache, sri, to)
}

pub fn copy_unchecked(cache: &Path, sri: &Integrity, to: &Path) -> Result<u64> {
    let cpath = path::content_path(cache, sri);
    std::fs::copy(cpath, to).with_context(|| {
        format!(
            "Failed to copy cache contents from {} to {}",
            path::content_path(cache, sri).display(),
            to.display()
        )
    })
}

pub fn copy(cache: &Path, sri: &Integrity, to: &Path) -> Result<u64> {
    let mut reader = open(cache, sri.clone())?;
    let mut buf: [u8; 1024] = [0; 1024];
    let mut size = 0;
    loop {
        let read = reader.read(&mut buf).with_context(|| {
            format!(
                "Failed to read cache contents while verifying integrity for {}",
                path::content_path(cache, sri).display()
            )
        })?;
        size += read;
        if read == 0 {
            break;
        }
    }
    reader.check()?;
    copy_unchecked(cache, sri, to)?;

    Ok(size as u64)
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn copy_unchecked_async<'a>(
    cache: &'a Path,
    sri: &'a Integrity,
    to: &'a Path,
) -> Result<u64> {
    let cpath = path::content_path(cache, sri);
    crate::async_lib::copy(&cpath, to).await.with_context(|| {
        format!(
            "Failed to copy cache contents from {} to {}",
            path::content_path(cache, sri).display(),
            to.display()
        )
    })
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn copy_async<'a>(cache: &'a Path, sri: &'a Integrity, to: &'a Path) -> Result<u64> {
    let mut reader = open_async(cache, sri.clone()).await?;
    let mut buf: [u8; 1024] = [0; 1024];
    let mut size = 0;
    loop {
        let read = AsyncReadExt::read(&mut reader, &mut buf)
            .await
            .with_context(|| {
                format!(
                    "Failed to read cache contents while verifying integrity for {}",
                    path::content_path(cache, sri).display()
                )
            })?;
        size += read;
        if read == 0 {
            break;
        }
    }
    reader.check()?;
    copy_unchecked_async(cache, sri, to).await?;
    Ok(size as u64)
}

pub fn hard_link_unchecked(cache: &Path, sri: &Integrity, to: &Path) -> Result<()> {
    let cpath = path::content_path(cache, sri);
    std::fs::hard_link(cpath, to).with_context(|| {
        format!(
            "Failed to link cache contents from {} to {}",
            path::content_path(cache, sri).display(),
            to.display()
        )
    })?;
    Ok(())
}

pub fn hard_link(cache: &Path, sri: &Integrity, to: &Path) -> Result<()> {
    hard_link_unchecked(cache, sri, to)?;
    let mut reader = open(cache, sri.clone())?;
    let mut buf = [0u8; 1024 * 8];
    loop {
        let read = reader.read(&mut buf).with_context(|| {
            format!(
                "Failed to read cache contents while verifying integrity for {}",
                path::content_path(cache, sri).display()
            )
        })?;
        if read == 0 {
            break;
        }
    }
    reader.check()?;
    Ok(())
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn hard_link_async(cache: &Path, sri: &Integrity, to: &Path) -> Result<()> {
    let mut reader = open_async(cache, sri.clone()).await?;
    let mut buf = [0u8; 1024 * 8];
    loop {
        let read = AsyncReadExt::read(&mut reader, &mut buf)
            .await
            .with_context(|| {
                format!(
                    "Failed to read cache contents while verifying integrity for {}",
                    path::content_path(cache, sri).display()
                )
            })?;
        if read == 0 {
            break;
        }
    }
    reader.check()?;
    hard_link_unchecked(cache, sri, to)?;
    Ok(())
}

pub fn has_content(cache: &Path, sri: &Integrity) -> Option<Integrity> {
    if path::content_path(cache, sri).exists() {
        Some(sri.clone())
    } else {
        None
    }
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn has_content_async(cache: &Path, sri: &Integrity) -> Option<Integrity> {
    if crate::async_lib::metadata(path::content_path(cache, sri))
        .await
        .is_ok()
    {
        Some(sri.clone())
    } else {
        None
    }
}
