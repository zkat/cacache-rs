use std::fs::DirBuilder;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
#[cfg(any(feature = "async-std", feature = "tokio"))]
use std::pin::Pin;
#[cfg(any(feature = "async-std", feature = "tokio"))]
use std::sync::Mutex;
#[cfg(any(feature = "async-std", feature = "tokio"))]
use std::task::{Context, Poll};

#[cfg(any(feature = "async-std", feature = "tokio"))]
use futures::prelude::*;
#[cfg(feature = "mmap")]
use memmap2::MmapMut;
use ssri::{Algorithm, Integrity, IntegrityOpts};
use tempfile::NamedTempFile;

#[cfg(any(feature = "async-std", feature = "tokio"))]
use crate::async_lib::{AsyncWrite, JoinHandle};
use crate::content::path;
use crate::errors::{IoErrorExt, Result};
use crate::Error;

#[cfg(feature = "mmap")]
pub const MAX_MMAP_SIZE: usize = 1024 * 1024;

#[cfg(not(feature = "mmap"))]
struct MmapMut;

#[cfg(not(feature = "mmap"))]
impl MmapMut {
    #[allow(dead_code)]
    fn flush_async(&self) -> std::io::Result<()> {
        panic!()
    }

    fn copy_from_slice(&self, _: &[u8]) {
        panic!()
    }
}

pub struct Writer {
    cache: PathBuf,
    builder: IntegrityOpts,
    mmap: Option<MmapMut>,
    tmpfile: NamedTempFile,
}

impl Writer {
    pub fn new(cache: &Path, algo: Algorithm, size: Option<usize>) -> Result<Writer> {
        let cache_path = cache.to_path_buf();
        let mut tmp_path = cache_path.clone();
        tmp_path.push("tmp");
        DirBuilder::new()
            .recursive(true)
            .create(&tmp_path)
            .with_context(|| {
                format!(
                    "Failed to create cache directory for temporary files, at {}",
                    tmp_path.display()
                )
            })?;
        let tmp_path_clone = tmp_path.clone();
        let mut tmpfile = NamedTempFile::new_in(tmp_path).with_context(|| {
            format!(
                "Failed to create temp file while initializing a writer, inside {}",
                tmp_path_clone.display()
            )
        })?;
        let mmap = make_mmap(&mut tmpfile, size)?;
        Ok(Writer {
            cache: cache_path,
            builder: IntegrityOpts::new().algorithm(algo),
            tmpfile,
            mmap,
        })
    }

    pub fn close(self) -> Result<Integrity> {
        let sri = self.builder.result();
        let cpath = path::content_path(&self.cache, &sri);
        DirBuilder::new()
            .recursive(true)
            // Safe unwrap. cpath always has multiple segments
            .create(cpath.parent().unwrap())
            .with_context(|| {
                format!(
                    "Failed to create destination directory for cache contents, at {}",
                    path::content_path(&self.cache, &sri)
                        .parent()
                        .unwrap()
                        .display()
                )
            })?;
        let res = self.tmpfile.persist(&cpath);
        match res {
            Ok(_) => {}
            Err(e) => {
                // We might run into conflicts sometimes when persisting files.
                // This is ok. We can deal. Let's just make sure the destination
                // file actually exists, and we can move on.
                if !cpath.exists() {
                    return Err(e.error).with_context(|| {
                        format!(
                            "Failed to persist cache contents while closing writer, at {}",
                            path::content_path(&self.cache, &sri).display()
                        )
                    })?;
                }
            }
        }
        Ok(sri)
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.builder.input(buf);
        if let Some(mmap) = &mut self.mmap {
            mmap.copy_from_slice(buf);
            Ok(buf.len())
        } else {
            self.tmpfile.write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.tmpfile.flush()
    }
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
pub struct AsyncWriter(Mutex<State>);

#[cfg(any(feature = "async-std", feature = "tokio"))]
enum State {
    Idle(Option<Inner>),
    Busy(JoinHandle<State>),
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
struct Inner {
    cache: PathBuf,
    builder: IntegrityOpts,
    tmpfile: NamedTempFile,
    mmap: Option<MmapMut>,
    buf: Vec<u8>,
    last_op: Option<Operation>,
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
enum Operation {
    Write(std::io::Result<usize>),
    Flush(std::io::Result<()>),
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
impl AsyncWriter {
    #[allow(clippy::new_ret_no_self)]
    #[allow(clippy::needless_lifetimes)]
    pub async fn new(cache: &Path, algo: Algorithm, size: Option<usize>) -> Result<AsyncWriter> {
        let cache_path = cache.to_path_buf();
        let mut tmp_path = cache_path.clone();
        tmp_path.push("tmp");
        crate::async_lib::DirBuilder::new()
            .recursive(true)
            .create(&tmp_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to create cache directory for temporary files, at {}",
                    tmp_path.display()
                )
            })?;

        match crate::async_lib::create_named_tempfile(tmp_path).await {
            Some(tmpfile) => {
                let mut tmpfile = tmpfile?;
                let mmap = make_mmap(&mut tmpfile, size)?;
                Ok(AsyncWriter(Mutex::new(State::Idle(Some(Inner {
                    cache: cache_path,
                    builder: IntegrityOpts::new().algorithm(algo),
                    mmap,
                    tmpfile,
                    buf: vec![],
                    last_op: None,
                })))))
            }
            _ => Err(Error::IoError(
                std::io::Error::other("File not created"),
                "Possible memory issues for file handle".into(),
            )),
        }
    }

    pub async fn close(self) -> Result<Integrity> {
        // NOTE: How do I even get access to `inner` safely???
        // let inner = ???;
        // Blocking, but should be a very fast op.
        futures::future::poll_fn(|cx| {
            let state = &mut *self.0.lock().unwrap();

            loop {
                match state {
                    State::Idle(opt) => match opt.take() {
                        None => return Poll::Ready(None),
                        Some(inner) => {
                            let (s, r) = futures::channel::oneshot::channel();
                            let tmpfile = inner.tmpfile;
                            let sri = inner.builder.result();
                            let cpath = path::content_path(&inner.cache, &sri);

                            // Start the operation asynchronously.
                            *state = State::Busy(crate::async_lib::spawn_blocking(|| {
                                let res = std::fs::DirBuilder::new()
                                    .recursive(true)
                                    // Safe unwrap. cpath always has multiple segments
                                    .create(cpath.parent().unwrap())
                                    .with_context(|| {
                                        format!(
                                            "building directory {} failed",
                                            cpath.parent().unwrap().display()
                                        )
                                    });
                                if res.is_err() {
                                    let _ = s.send(res.map(|_| sri));
                                } else {
                                    let res = tmpfile
                                        .persist(&cpath)
                                        .map_err(|e| e.error)
                                        .with_context(|| {
                                            format!("persisting file {} failed", cpath.display())
                                        });
                                    if res.is_err() {
                                        // We might run into conflicts
                                        // sometimes when persisting files.
                                        // This is ok. We can deal. Let's just
                                        // make sure the destination file
                                        // actually exists, and we can move
                                        // on.
                                        let _ = s.send(
                                            std::fs::metadata(cpath)
                                                .with_context(|| {
                                                    String::from("File still doesn't exist")
                                                })
                                                .map(|_| sri),
                                        );
                                    } else {
                                        let _ = s.send(res.map(|_| sri));
                                    }
                                }
                                State::Idle(None)
                            }));

                            return Poll::Ready(Some(r));
                        }
                    },
                    // Poll the asynchronous operation the file is currently blocked on.
                    State::Busy(task) => {
                        let next_state = crate::async_lib::unwrap_joinhandle_value(
                            futures::ready!(Pin::new(task).poll(cx)),
                        );

                        update_state(state, next_state);
                    }
                }
            }
        })
        .map(|opt| opt.ok_or_else(|| crate::errors::io_error("file closed")))
        .await
        .with_context(|| "Error while closing cache contents".to_string())?
        .await
        .map_err(|_| crate::errors::io_error("Operation cancelled"))
        .with_context(|| "Error while closing cache contents".to_string())?
    }
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
impl AsyncWrite for AsyncWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.0.lock() {
            Ok(mut state) => {
                let state = &mut *state;

                loop {
                    match state {
                        State::Idle(opt) => {
                            // Grab a reference to the inner representation of the file or return an error
                            // if the file is closed.
                            let inner = opt
                                .as_mut()
                                .ok_or_else(|| crate::errors::io_error("file closed"))?;

                            // Check if the operation has completed.
                            if let Some(Operation::Write(res)) = inner.last_op.take() {
                                let n = res?;

                                // If more data was written than is available in the buffer, let's retry
                                // the write operation.
                                if n <= buf.len() {
                                    return Poll::Ready(Ok(n));
                                }
                            } else {
                                let mut inner = opt.take().unwrap();

                                // Set the length of the inner buffer to the length of the provided buffer.
                                if inner.buf.len() < buf.len() {
                                    inner.buf.reserve(buf.len() - inner.buf.len());
                                }
                                unsafe {
                                    inner.buf.set_len(buf.len());
                                }

                                // Copy the data to write into the inner buffer.
                                inner.buf[..buf.len()].copy_from_slice(buf);

                                // Start the operation asynchronously.
                                *state = State::Busy(crate::async_lib::spawn_blocking(|| {
                                    inner.builder.input(&inner.buf);
                                    if let Some(mmap) = &mut inner.mmap {
                                        mmap.copy_from_slice(&inner.buf);
                                        inner.last_op = Some(Operation::Write(Ok(inner.buf.len())));
                                        State::Idle(Some(inner))
                                    } else {
                                        let res = inner.tmpfile.write(&inner.buf);
                                        inner.last_op = Some(Operation::Write(res));
                                        State::Idle(Some(inner))
                                    }
                                }));
                            }
                        }
                        // Poll the asynchronous operation the file is currently blocked on.
                        State::Busy(task) => {
                            let next_state = crate::async_lib::unwrap_joinhandle_value(
                                futures::ready!(Pin::new(task).poll(cx)),
                            );

                            update_state(state, next_state);
                        }
                    }
                }
            }
            _ => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.0.lock() {
            Ok(mut state) => {
                let state = &mut *state;
                loop {
                    match state {
                        State::Idle(opt) => {
                            // Grab a reference to the inner representation of the file or return if the
                            // file is closed.
                            let inner = match opt.as_mut() {
                                None => return Poll::Ready(Ok(())),
                                Some(s) => s,
                            };

                            // Check if the operation has completed.
                            if let Some(Operation::Flush(res)) = inner.last_op.take() {
                                return Poll::Ready(res);
                            } else {
                                let mut inner = opt.take().unwrap();

                                if let Some(mmap) = &inner.mmap {
                                    match mmap.flush_async() {
                                        Ok(_) => (),
                                        Err(e) => return Poll::Ready(Err(e)),
                                    };
                                }

                                // Start the operation asynchronously.
                                *state = State::Busy(crate::async_lib::spawn_blocking(|| {
                                    let res = inner.tmpfile.flush();
                                    inner.last_op = Some(Operation::Flush(res));
                                    State::Idle(Some(inner))
                                }));
                            }
                        }
                        // Poll the asynchronous operation the file is currently blocked on.
                        State::Busy(task) => {
                            let next_state = crate::async_lib::unwrap_joinhandle_value(
                                futures::ready!(Pin::new(task).poll(cx)),
                            );

                            update_state(state, next_state);
                        }
                    }
                }
            }
            _ => Poll::Pending,
        }
    }

    #[cfg(feature = "async-std")]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.poll_close_impl(cx)
    }

    #[cfg(feature = "tokio")]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.poll_close_impl(cx)
    }
}

#[cfg(feature = "tokio")]
/// Update the state.
fn update_state(
    current_state: &mut State,
    next_state: std::result::Result<State, tokio::task::JoinError>,
) {
    match next_state {
        Ok(next) => {
            *current_state = next;
        }
        _ => {
            *current_state = State::Idle(None);
        }
    }
}

#[cfg(not(feature = "tokio"))]
/// Update the state.
fn update_state(current_state: &mut State, next_state: State) {
    *current_state = next_state;
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
impl AsyncWriter {
    #[inline]
    fn poll_close_impl(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.0.lock() {
            Ok(mut state) => {
                let state = &mut *state;
                loop {
                    match state {
                        State::Idle(opt) => {
                            // Grab a reference to the inner representation of the file or return if the
                            // file is closed.
                            let inner = match opt.take() {
                                None => return Poll::Ready(Ok(())),
                                Some(s) => s,
                            };

                            // Start the operation asynchronously.
                            *state = State::Busy(crate::async_lib::spawn_blocking(|| {
                                drop(inner);
                                State::Idle(None)
                            }));
                        }
                        // Poll the asynchronous operation the file is currently blocked on.
                        State::Busy(task) => {
                            let next_state = crate::async_lib::unwrap_joinhandle_value(
                                futures::ready!(Pin::new(task).poll(cx)),
                            );

                            update_state(state, next_state);
                        }
                    }
                }
            }
            _ => Poll::Pending,
        }
    }
}

#[cfg(feature = "mmap")]
fn make_mmap(tmpfile: &mut NamedTempFile, size: Option<usize>) -> Result<Option<MmapMut>> {
    if let Some(size @ 0..=MAX_MMAP_SIZE) = size {
        allocate_file(tmpfile.as_file(), size).with_context(|| {
            format!(
                "Failed to configure file length for temp file at {}",
                tmpfile.path().display()
            )
        })?;
        Ok(unsafe { MmapMut::map_mut(tmpfile.as_file()).ok() })
    } else {
        Ok(None)
    }
}

#[cfg(feature = "mmap")]
#[cfg(target_os = "linux")]
fn allocate_file(file: &std::fs::File, size: usize) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};
    use std::os::fd::AsRawFd;

    let fd = file.as_raw_fd();
    match unsafe { libc::posix_fallocate64(fd, 0, size as i64) } {
        0 => Ok(()),
        libc::ENOSPC => Err(Error::new(
            ErrorKind::Other, // ErrorKind::StorageFull is unstable
            "cannot allocate file: no space left on device",
        )),
        err => Err(Error::new(
            ErrorKind::Other,
            format!("posix_fallocate64 failed with code {err}"),
        )),
    }
}

#[cfg(feature = "mmap")]
#[cfg(not(target_os = "linux"))]
fn allocate_file(file: &std::fs::File, size: usize) -> std::io::Result<()> {
    file.set_len(size as u64)
}

#[cfg(not(feature = "mmap"))]
fn make_mmap(_: &mut NamedTempFile, _: Option<usize>) -> Result<Option<MmapMut>> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(feature = "async-std", feature = "tokio"))]
    use crate::async_lib::AsyncWriteExt;
    use tempfile;

    #[cfg(feature = "async-std")]
    use async_attributes::test as async_test;
    #[cfg(feature = "tokio")]
    use tokio::test as async_test;

    #[test]
    fn basic_write() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut writer = Writer::new(&dir, Algorithm::Sha256, None).unwrap();
        writer.write_all(b"hello world").unwrap();
        let sri = writer.close().unwrap();
        assert_eq!(sri.to_string(), Integrity::from(b"hello world").to_string());
        assert_eq!(
            std::fs::read(path::content_path(&dir, &sri)).unwrap(),
            b"hello world"
        );
    }

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    #[async_test]
    async fn basic_async_write() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut writer = AsyncWriter::new(&dir, Algorithm::Sha256, None)
            .await
            .unwrap();
        writer.write_all(b"hello world").await.unwrap();
        let sri = writer.close().await.unwrap();
        assert_eq!(sri.to_string(), Integrity::from(b"hello world").to_string());
        assert_eq!(
            std::fs::read(path::content_path(&dir, &sri)).unwrap(),
            b"hello world"
        );
    }
}
