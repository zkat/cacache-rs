#[cfg(feature = "async-std")]
pub use async_std::fs::File;
#[cfg(feature = "tokio")]
pub use tokio::fs::File;

#[cfg(feature = "async-std")]
pub use futures::io::AsyncRead;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncRead;

#[cfg(feature = "async-std")]
pub use futures::io::AsyncReadExt;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncReadExt;

#[cfg(feature = "async-std")]
pub use futures::io::AsyncBufReadExt;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncBufReadExt;

#[cfg(feature = "async-std")]
pub use futures::io::AsyncWrite;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncWrite;

#[cfg(feature = "async-std")]
pub use futures::io::AsyncWriteExt;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncWriteExt;

#[cfg(feature = "async-std")]
pub use async_std::fs::read;
#[cfg(feature = "tokio")]
pub use tokio::fs::read;

#[cfg(feature = "async-std")]
pub use async_std::fs::copy;
#[cfg(feature = "tokio")]
pub use tokio::fs::copy;

#[cfg(feature = "async-std")]
pub use async_std::fs::metadata;
#[cfg(feature = "tokio")]
pub use tokio::fs::metadata;

#[cfg(feature = "async-std")]
pub use async_std::fs::remove_file;
#[cfg(feature = "tokio")]
pub use tokio::fs::remove_file;

#[cfg(feature = "async-std")]
pub use async_std::fs::create_dir_all;
#[cfg(feature = "tokio")]
pub use tokio::fs::create_dir_all;

#[cfg(feature = "async-std")]
pub use async_std::fs::remove_dir_all;
#[cfg(feature = "tokio")]
pub use tokio::fs::remove_dir_all;

#[cfg(feature = "async-std")]
pub use async_std::fs::DirBuilder;
#[cfg(feature = "tokio")]
pub use tokio::fs::DirBuilder;

#[cfg(feature = "async-std")]
pub use async_std::fs::OpenOptions;
#[cfg(feature = "tokio")]
pub use tokio::fs::OpenOptions;

#[cfg(feature = "async-std")]
pub use async_std::io::BufReader;
#[cfg(feature = "tokio")]
pub use tokio::io::BufReader;

#[cfg(feature = "async-std")]
#[inline]
pub fn lines_to_stream<R>(lines: futures::io::Lines<R>) -> futures::io::Lines<R> {
    lines
}
#[cfg(feature = "tokio")]
#[inline]
pub fn lines_to_stream<R>(lines: tokio::io::Lines<R>) -> tokio_stream::wrappers::LinesStream<R> {
    tokio_stream::wrappers::LinesStream::new(lines)
}

#[cfg(feature = "async-std")]
pub use async_std::task::spawn_blocking;
#[cfg(feature = "tokio")]
pub use tokio::task::spawn_blocking;

#[cfg(feature = "async-std")]
pub use async_std::task::JoinHandle;
#[cfg(feature = "async-std")]
#[inline]
pub fn unwrap_joinhandle_value<T>(value: T) -> T {
    value
}
#[cfg(feature = "tokio")]
pub use tokio::task::JoinHandle;
#[cfg(feature = "tokio")]
#[inline]
pub fn unwrap_joinhandle_value<T>(value: Result<T, tokio::task::JoinError>) -> T {
    value.unwrap()
}

use crate::errors::{Internal, InternalResult};
use tempfile::NamedTempFile;

#[cfg(feature = "async-std")]
#[inline]
pub async fn create_named_tempfile(tmp_path: std::path::PathBuf) -> InternalResult<NamedTempFile> {
    spawn_blocking(|| NamedTempFile::new_in(tmp_path))
        .await
        .to_internal()
}

#[cfg(feature = "tokio")]
#[inline]
pub async fn create_named_tempfile(tmp_path: std::path::PathBuf) -> InternalResult<NamedTempFile> {
    let tmpfile = spawn_blocking(|| NamedTempFile::new_in(tmp_path))
        .await
        .to_internal()?;
    tmpfile.to_internal()
}
