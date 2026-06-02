#[cfg(feature = "smol")]
pub use smol::fs::File;
#[cfg(feature = "tokio")]
pub use tokio::fs::File;

#[cfg(feature = "smol")]
pub use futures::io::AsyncRead;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncRead;

#[cfg(feature = "smol")]
pub use futures::io::AsyncReadExt;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncReadExt;

#[cfg(feature = "smol")]
pub use futures::io::AsyncBufReadExt;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncBufReadExt;

#[cfg(feature = "smol")]
pub use futures::io::AsyncWrite;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncWrite;

#[cfg(feature = "smol")]
pub use futures::io::AsyncWriteExt;
#[cfg(feature = "tokio")]
pub use tokio::io::AsyncWriteExt;

#[cfg(feature = "smol")]
pub use smol::fs::read;
#[cfg(feature = "tokio")]
pub use tokio::fs::read;

#[cfg(feature = "smol")]
pub use smol::fs::copy;
#[cfg(feature = "tokio")]
pub use tokio::fs::copy;

#[cfg(feature = "smol")]
pub use smol::fs::metadata;
#[cfg(feature = "tokio")]
pub use tokio::fs::metadata;

#[cfg(feature = "smol")]
pub use smol::fs::remove_file;
#[cfg(feature = "tokio")]
pub use tokio::fs::remove_file;

#[cfg(feature = "smol")]
pub use smol::fs::create_dir_all;
#[cfg(feature = "tokio")]
pub use tokio::fs::create_dir_all;

#[cfg(feature = "smol")]
pub use smol::fs::remove_dir_all;
#[cfg(feature = "tokio")]
pub use tokio::fs::remove_dir_all;

#[cfg(feature = "smol")]
pub use smol::fs::DirBuilder;
#[cfg(feature = "tokio")]
pub use tokio::fs::DirBuilder;

#[cfg(feature = "smol")]
pub use smol::fs::OpenOptions;
#[cfg(feature = "tokio")]
pub use tokio::fs::OpenOptions;

#[cfg(feature = "smol")]
pub use futures::io::BufReader;
#[cfg(feature = "tokio")]
pub use tokio::io::BufReader;

#[cfg(feature = "smol")]
#[inline]
pub fn lines_to_stream<R>(lines: futures::io::Lines<R>) -> futures::io::Lines<R> {
    lines
}
#[cfg(feature = "tokio")]
#[inline]
pub fn lines_to_stream<R>(lines: tokio::io::Lines<R>) -> tokio_stream::wrappers::LinesStream<R> {
    tokio_stream::wrappers::LinesStream::new(lines)
}

#[cfg(feature = "smol")]
pub use smol::unblock as spawn_blocking;
#[cfg(feature = "tokio")]
pub use tokio::task::spawn_blocking;

#[cfg(feature = "smol")]
pub use smol::Task as JoinHandle;
#[cfg(feature = "tokio")]
pub use tokio::task::JoinHandle;

#[cfg(feature = "smol")]
#[inline]
pub fn unwrap_joinhandle_value<T>(value: T) -> T {
    value
}
#[cfg(feature = "tokio")]
#[inline]
pub fn unwrap_joinhandle_value<T>(value: T) -> T {
    value
}

use crate::errors::IoErrorExt;
use tempfile::NamedTempFile;

#[cfg(feature = "smol")]
#[inline]
pub async fn create_named_tempfile(
    tmp_path: std::path::PathBuf,
) -> Option<crate::Result<NamedTempFile>> {
    let cloned = tmp_path.clone();

    Some(
        spawn_blocking(|| NamedTempFile::new_in(tmp_path))
            .await
            .with_context(|| format!("Failed to create a temp file at {}", cloned.display())),
    )
}

#[cfg(feature = "tokio")]
#[inline]
pub async fn create_named_tempfile(
    tmp_path: std::path::PathBuf,
) -> Option<crate::Result<NamedTempFile>> {
    let cloned = tmp_path.clone();
    match spawn_blocking(|| NamedTempFile::new_in(tmp_path)).await {
        Ok(ctx) => Some(
            ctx.with_context(|| format!("Failed to create a temp file at {}", cloned.display())),
        ),
        _ => None,
    }
}
