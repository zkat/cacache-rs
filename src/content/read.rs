use std::fs::{self, File};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::Result;
use async_std;
use futures::prelude::*;
use ssri::{Algorithm, Integrity, IntegrityChecker};

use crate::content::path;
use crate::errors::Error;

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
        self.checker
            .result()
            .ok_or(anyhow::Error::new(Error::IntegrityError))
    }
}

pub struct AsyncReader {
    fd: async_std::fs::File,
    checker: IntegrityChecker,
}

impl AsyncRead for AsyncReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let amt = futures::ready!(Pin::new(&mut self.fd).poll_read(cx, buf))?;
        self.checker.input(&buf[..amt]);
        Poll::Ready(Ok(amt))
    }
}

impl AsyncReader {
    pub fn check(self) -> Result<Algorithm> {
        self.checker
            .result()
            .ok_or(anyhow::Error::new(Error::IntegrityError))
    }
}

pub fn open(cache: &Path, sri: Integrity) -> Result<Reader> {
    let cpath = path::content_path(&cache, &sri);
    Ok(Reader {
        fd: File::open(cpath)?,
        checker: IntegrityChecker::new(sri),
    })
}

pub async fn open_async(cache: &Path, sri: Integrity) -> Result<AsyncReader> {
    let cpath = path::content_path(&cache, &sri);
    Ok(AsyncReader {
        fd: async_std::fs::File::open(cpath).await?,
        checker: IntegrityChecker::new(sri),
    })
}

pub fn read(cache: &Path, sri: &Integrity) -> Result<Vec<u8>> {
    let cpath = path::content_path(&cache, &sri);
    let ret = fs::read(&cpath)?;
    if sri.check(&ret).is_some() {
        Ok(ret)
    } else {
        Err(Error::IntegrityError)?
    }
}

pub async fn read_async<'a>(cache: &'a Path, sri: &'a Integrity) -> Result<Vec<u8>> {
    let cpath = path::content_path(&cache, &sri);
    let ret = async_std::fs::read(&cpath).await?;
    if sri.check(&ret).is_some() {
        Ok(ret)
    } else {
        Err(Error::IntegrityError)?
    }
}

pub fn copy(cache: &Path, sri: &Integrity, to: &Path) -> Result<u64> {
    let cpath = path::content_path(&cache, &sri);
    let ret = fs::copy(&cpath, to)?;
    let data = fs::read(cpath)?;
    if sri.check(data).is_some() {
        Ok(ret)
    } else {
        Err(Error::IntegrityError)?
    }
}

pub async fn copy_async<'a>(cache: &'a Path, sri: &'a Integrity, to: &'a Path) -> Result<u64> {
    let cpath = path::content_path(&cache, &sri);
    let ret = async_std::fs::copy(&cpath, to).await?;
    let data = async_std::fs::read(cpath).await?;
    if sri.check(data).is_some() {
        Ok(ret)
    } else {
        Err(Error::IntegrityError)?
    }
}

pub fn has_content(cache: &Path, sri: &Integrity) -> Option<Integrity> {
    if path::content_path(&cache, &sri).exists() {
        Some(sri.clone())
    } else {
        None
    }
}

pub async fn has_content_async(cache: &Path, sri: &Integrity) -> Option<Integrity> {
    if async_std::fs::metadata(path::content_path(&cache, &sri))
        .await
        .is_ok()
    {
        Some(sri.clone())
    } else {
        None
    }
}
