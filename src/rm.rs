//! Functions for removing things from the cache.
use std::fs;
use std::path::Path;

use async_std::fs as afs;

use anyhow::{Context, Result};
use ssri::Integrity;

use crate::content::rm;
use crate::index;

/// Removes an individual index entry. The associated content will be left
/// intact.
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
/// let sri = cacache::put::data("./my-cache", "my-key", b"hello").await?;
///
/// cacache::rm::entry("./my-cache", "my-key").await?;
///
/// // This fails:
/// cacache::get::data("./my-cache", "my-key").await?;
///
/// // But this succeeds:
/// cacache::get::data_hash("./my-cache", &sri).await?;
/// # Ok(())
/// # }
/// ```
pub async fn entry<P, K>(cache: P, key: K) -> Result<()>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    index::delete_async(cache.as_ref(), key.as_ref())
        .await
        .with_context(|| {
            format!(
                "Failed to delete cache entry for {} in cache at {:?}",
                key.as_ref(),
                cache.as_ref()
            )
        })
}

/// Removes an individual content entry. Any index entries pointing to this
/// content will become invalidated.
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
/// let sri = cacache::put::data("./my-cache", "my-key", b"hello").await?;
///
/// cacache::rm::entry("./my-cache", "my-key").await?;
///
/// // These fail:
/// cacache::get::data("./my-cache", "my-key").await?;
/// cacache::get::data_hash("./my-cache", &sri).await?;
///
/// // But this succeeds:
/// cacache::get::entry("./my-cache", "my-key").await?;
/// # Ok(())
/// # }
/// ```
pub async fn content<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<()> {
    rm::rm_async(cache.as_ref(), &sri).await.with_context(|| {
        format!(
            "Failed to remove content under {} in cache at {:?}",
            sri.to_string(),
            cache.as_ref()
        )
    })
}

/// Removes entire contents of the cache, including temporary files, the entry
/// index, and all content data.
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
/// let sri = cacache::put::data("./my-cache", "my-key", b"hello").await?;
///
/// cacache::rm::entry("./my-cache", "my-key").await?;
///
/// // These all fail:
/// cacache::get::data("./my-cache", "my-key").await?;
/// cacache::get::entry("./my-cache", "my-key").await?;
/// cacache::get::data_hash("./my-cache", &sri).await?;
/// # Ok(())
/// # }
/// ```
pub async fn all<P: AsRef<Path>>(cache: P) -> Result<()> {
    for entry in cache.as_ref().read_dir()? {
        if let Ok(entry) = entry {
            afs::remove_dir_all(entry.path()).await?;
        }
    }
    Ok(())
}

/// Removes an individual index entry synchronously. The associated content
/// will be left intact.
///
/// ## Example
/// ```no_run
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// # use std::io::Read;
/// let sri = cacache::put::data_sync("./my-cache", "my-key", b"hello")?;
///
/// cacache::rm::entry_sync("./my-cache", "my-key")?;
///
/// // This fails:
/// cacache::get::data_sync("./my-cache", "my-key")?;
///
/// // But this succeeds:
/// cacache::get::data_hash_sync("./my-cache", &sri)?;
/// # Ok(())
/// # }
/// ```
pub fn entry_sync<P, K>(cache: P, key: K) -> Result<()>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    index::delete(cache.as_ref(), key.as_ref()).with_context(|| {
        format!(
            "Failed to delete cache entry for {} in cache at {:?}",
            key.as_ref(),
            cache.as_ref()
        )
    })
}

/// Removes an individual content entry synchronously. Any index entries
/// pointing to this content will become invalidated.
///
/// ## Example
/// ```no_run
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// # use std::io::Read;
/// let sri = cacache::put::data_sync("./my-cache", "my-key", b"hello")?;
///
/// cacache::rm::entry_sync("./my-cache", "my-key")?;
///
/// // These fail:
/// cacache::get::data_sync("./my-cache", "my-key")?;
/// cacache::get::data_hash_sync("./my-cache", &sri)?;
///
/// // But this succeeds:
/// cacache::get::entry_sync("./my-cache", "my-key")?;
/// # Ok(())
/// # }
/// ```
pub fn content_sync<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<()> {
    rm::rm(cache.as_ref(), &sri).with_context(|| {
        format!(
            "Failed to remove content under {} in cache at {:?}",
            sri.to_string(),
            cache.as_ref()
        )
    })
}

/// Removes entire contents of the cache synchronously, including temporary
/// files, the entry index, and all content data.
///
/// ## Example
/// ```no_run
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// # use std::io::Read;
/// let sri = cacache::put::data_sync("./my-cache", "my-key", b"hello")?;
///
/// cacache::rm::entry_sync("./my-cache", "my-key")?;
///
/// // These all fail:
/// cacache::get::data_sync("./my-cache", "my-key")?;
/// cacache::get::data_hash_sync("./my-cache", &sri)?;
/// cacache::get::entry_sync("./my-cache", "my-key")?;
/// # Ok(())
/// # }
/// ```
pub fn all_sync<P: AsRef<Path>>(cache: P) -> Result<()> {
    for entry in cache.as_ref().read_dir()? {
        if let Ok(entry) = entry {
            fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_std::task;

    #[test]
    fn entry() {
        task::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            let sri = crate::put::data(&dir, "key", b"my-data").await.unwrap();

            crate::rm::entry(&dir, "key").await.unwrap();

            let entry = crate::get::entry(&dir, "key").await.unwrap();
            assert_eq!(entry, None);

            let data_exists = crate::get::hash_exists(&dir, &sri).await;
            assert_eq!(data_exists, true);
        });
    }

    #[test]
    fn content() {
        task::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            let sri = crate::put::data(&dir, "key", b"my-data").await.unwrap();

            crate::rm::content(&dir, &sri).await.unwrap();

            let entry = crate::get::entry(&dir, "key").await.unwrap();
            assert_eq!(entry.is_some(), true);

            let data_exists = crate::get::hash_exists(&dir, &sri).await;
            assert_eq!(data_exists, false);
        });
    }

    #[test]
    fn all() {
        task::block_on(async {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().to_owned();
            let sri = crate::put::data(&dir, "key", b"my-data").await.unwrap();

            crate::rm::all(&dir).await.unwrap();

            let entry = crate::get::entry(&dir, "key").await.unwrap();
            assert_eq!(entry.is_some(), false);

            let data_exists = crate::get::hash_exists(&dir, &sri).await;
            assert_eq!(data_exists, false);
        });
    }

    #[test]
    fn entry_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::put::data_sync(&dir, "key", b"my-data").unwrap();

        crate::rm::entry_sync(&dir, "key").unwrap();

        let new_entry = crate::get::entry_sync(&dir, "key").unwrap();
        assert_eq!(new_entry, None);

        let data_exists = crate::get::hash_exists_sync(&dir, &sri);
        assert_eq!(data_exists, true);
    }

    #[test]
    fn content_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::put::data_sync(&dir, "key", b"my-data").unwrap();

        crate::rm::content_sync(&dir, &sri).unwrap();

        let new_entry = crate::get::entry_sync(&dir, "key").unwrap();
        assert_eq!(new_entry.is_some(), true);

        let data_exists = crate::get::hash_exists_sync(&dir, &sri);
        assert_eq!(data_exists, false);
    }

    #[test]
    fn all_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::put::data_sync(&dir, "key", b"my-data").unwrap();

        crate::rm::all_sync(&dir).unwrap();

        let new_entry = crate::get::entry_sync(&dir, "key").unwrap();
        assert_eq!(new_entry, None);

        let data_exists = crate::get::hash_exists_sync(&dir, &sri);
        assert_eq!(data_exists, false);
    }
}
