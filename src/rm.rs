//! Functions for removing things from the cache.
use std::fs;
use std::path::Path;

use ssri::Integrity;

use crate::content::rm;
use crate::errors::{IoErrorExt, Result};
use crate::index;

/// Removes an individual index metadata entry. The associated content will be
/// left in the cache.
///
/// ## Example
/// ```no_run
/// use async_std::prelude::*;
/// use async_attributes;
///
/// #[async_attributes::main]
/// async fn main() -> cacache::Result<()> {
///     let sri = cacache::write("./my-cache", "my-key", b"hello").await?;
///
///     cacache::remove("./my-cache", "my-key").await?;
///
///     // This fails:
///     cacache::read("./my-cache", "my-key").await?;
///
///     // But this succeeds:
///     cacache::read_hash("./my-cache", &sri).await?;
///
///     Ok(())
/// }
/// ```
#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn remove<P, K>(cache: P, key: K) -> Result<()>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    index::delete_async(cache.as_ref(), key.as_ref()).await
}

/// Removes an individual content entry. Any index entries pointing to this
/// content will become invalidated.
///
/// ## Example
/// ```no_run
/// use async_std::prelude::*;
/// use async_attributes;
///
/// #[async_attributes::main]
/// async fn main() -> cacache::Result<()> {
///     let sri = cacache::write("./my-cache", "my-key", b"hello").await?;
///
///     cacache::remove_hash("./my-cache", &sri).await?;
///
///     // These fail:
///     cacache::read("./my-cache", "my-key").await?;
///     cacache::read_hash("./my-cache", &sri).await?;
///
///     // But this succeeds:
///     cacache::metadata("./my-cache", "my-key").await?;
///
///     Ok(())
/// }
/// ```
#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn remove_hash<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<()> {
    rm::rm_async(cache.as_ref(), sri).await
}

/// Removes entire contents of the cache, including temporary files, the entry
/// index, and all content data.
///
/// ## Example
/// ```no_run
/// use async_std::prelude::*;
/// use async_attributes;
///
/// #[async_attributes::main]
/// async fn main() -> cacache::Result<()> {
///     let sri = cacache::write("./my-cache", "my-key", b"hello").await?;
///
///     cacache::clear("./my-cache").await?;
///
///     // These all fail:
///     cacache::read("./my-cache", "my-key").await?;
///     cacache::metadata("./my-cache", "my-key").await?;
///     cacache::read_hash("./my-cache", &sri).await?;
///
///     Ok(())
/// }
/// ```
#[cfg(any(feature = "async-std", feature = "tokio"))]
pub async fn clear<P: AsRef<Path>>(cache: P) -> Result<()> {
    async fn inner(cache: &Path) -> Result<()> {
        for entry in cache
            .read_dir()
            .with_context(|| {
                format!(
                    "Failed to read directory contents while clearing cache, at {}",
                    cache.display()
                )
            })?
            .flatten()
        {
            crate::async_lib::remove_dir_all(entry.path())
                .await
                .with_context(|| format!("Failed to clear cache at {}", cache.display()))?;
        }
        Ok(())
    }
    inner(cache.as_ref()).await
}

/// Removes an individual index entry synchronously. The associated content
/// will be left in the cache.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache::Result<()> {
///     let sri = cacache::write_sync("./my-cache", "my-key", b"hello")?;
///
///     cacache::remove_sync("./my-cache", "my-key")?;
///
///     // This fails:
///     cacache::read_sync("./my-cache", "my-key")?;
///
///     // But this succeeds:
///     cacache::read_hash_sync("./my-cache", &sri)?;
///
///     Ok(())
/// }
/// ```
pub fn remove_sync<P, K>(cache: P, key: K) -> Result<()>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    index::delete(cache.as_ref(), key.as_ref())
}

/// Removes an individual content entry synchronously. Any index entries
/// pointing to this content will become invalidated.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache::Result<()> {
///     let sri = cacache::write_sync("./my-cache", "my-key", b"hello")?;
///
///     cacache::remove_hash_sync("./my-cache", &sri)?;
///
///     // These fail:
///     cacache::read_sync("./my-cache", "my-key")?;
///     cacache::read_hash_sync("./my-cache", &sri)?;
///
///     // But this succeeds:
///     cacache::metadata_sync("./my-cache", "my-key")?;
///
///     Ok(())
/// }
/// ```
pub fn remove_hash_sync<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<()> {
    rm::rm(cache.as_ref(), sri)
}

/// Removes entire contents of the cache synchronously, including temporary
/// files, the entry index, and all content data.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache::Result<()> {
///     let sri = cacache::write_sync("./my-cache", "my-key", b"hello")?;
///
///     cacache::clear_sync("./my-cache")?;
///
///     // These all fail:
///     cacache::read_sync("./my-cache", "my-key")?;
///     cacache::read_hash_sync("./my-cache", &sri)?;
///     cacache::metadata_sync("./my-cache", "my-key")?;
///
///     Ok(())
/// }
/// ```
pub fn clear_sync<P: AsRef<Path>>(cache: P) -> Result<()> {
    fn inner(cache: &Path) -> Result<()> {
        for entry in cache
            .read_dir()
            .with_context(|| {
                format!(
                    "Failed to read directory contents while clearing cache, at {}",
                    cache.display()
                )
            })?
            .flatten()
        {
            fs::remove_dir_all(entry.path())
                .with_context(|| format!("Failed to clear cache at {}", cache.display()))?;
        }
        Ok(())
    }
    inner(cache.as_ref())
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "async-std")]
    use async_attributes::test as async_test;
    #[cfg(feature = "tokio")]
    use tokio::test as async_test;

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    #[async_test]
    async fn test_remove() {
        futures::executor::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            let sri = crate::write(&dir, "key", b"my-data").await.unwrap();

            crate::remove(&dir, "key").await.unwrap();

            let entry = crate::metadata(&dir, "key").await.unwrap();
            assert_eq!(entry, None);

            let data_exists = crate::exists(&dir, &sri).await;
            assert!(data_exists);
        });
    }

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    #[async_test]
    async fn test_remove_data() {
        futures::executor::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            let sri = crate::write(&dir, "key", b"my-data").await.unwrap();

            crate::remove_hash(&dir, &sri).await.unwrap();

            let entry = crate::metadata(&dir, "key").await.unwrap();
            assert!(entry.is_some());

            let data_exists = crate::exists(&dir, &sri).await;
            assert!(!data_exists);
        });
    }

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    #[async_test]
    async fn test_clear() {
        futures::executor::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            let sri = crate::write(&dir, "key", b"my-data").await.unwrap();

            crate::clear(&dir).await.unwrap();

            let entry = crate::metadata(&dir, "key").await.unwrap();
            assert!(entry.is_none());

            let data_exists = crate::exists(&dir, &sri).await;
            assert!(!data_exists);
        });
    }

    #[test]
    fn test_remove_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::write_sync(&dir, "key", b"my-data").unwrap();

        crate::remove_sync(&dir, "key").unwrap();

        let new_entry = crate::metadata_sync(&dir, "key").unwrap();
        assert!(new_entry.is_none());

        let data_exists = crate::exists_sync(&dir, &sri);
        assert!(data_exists);
    }

    #[test]
    fn test_remove_data_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::write_sync(&dir, "key", b"my-data").unwrap();

        crate::remove_hash_sync(&dir, &sri).unwrap();

        let entry = crate::metadata_sync(&dir, "key").unwrap();
        assert!(entry.is_some());

        let data_exists = crate::exists_sync(&dir, &sri);
        assert!(!data_exists);
    }

    #[test]
    fn test_clear_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::write_sync(&dir, "key", b"my-data").unwrap();

        crate::clear_sync(&dir).unwrap();

        let entry = crate::metadata_sync(&dir, "key").unwrap();
        assert_eq!(entry, None);

        let data_exists = crate::exists_sync(&dir, &sri);
        assert!(!data_exists);
    }
}
