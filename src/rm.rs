//! Functions for removing things from the cache.
use std::fs;
use std::path::Path;

use async_std::fs as afs;

use ssri::Integrity;

use crate::content::rm;
use crate::errors::Error;
use crate::index;

/// Removes an individual index entry. The associated content will be left
/// intact.
pub async fn entry<P: AsRef<Path>>(cache: P, key: &str) -> Result<(), Error> {
    index::delete_async(cache.as_ref(), &key).await
}

/// Removes an individual content entry. Any index entries pointing to this
/// content will become invalidated.
pub async fn content<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<(), Error> {
    rm::rm_async(cache.as_ref(), &sri).await
}

/// Removes entire contents of the cache, including temporary files, the entry
/// index, and all content data.
pub async fn all<P: AsRef<Path>>(cache: P) -> Result<(), Error> {
    for entry in cache.as_ref().read_dir()? {
        if let Ok(entry) = entry {
            afs::remove_dir_all(entry.path()).await?;
        }
    }
    Ok(())
}

/// Removes an individual index entry synchronously. The associated content
/// will be left intact.
pub fn entry_sync<P: AsRef<Path>>(cache: P, key: &str) -> Result<(), Error> {
    index::delete(cache.as_ref(), &key)
}

/// Removes an individual content entry synchronously. Any index entries
/// pointing to this content will become invalidated.
pub fn content_sync<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<(), Error> {
    rm::rm(cache.as_ref(), &sri)
}

/// Removes entire contents of the cache synchronously, including temporary files, the entry
/// index, and all content data.
pub fn all_sync<P: AsRef<Path>>(cache: P) -> Result<(), Error> {
    for entry in cache.as_ref().read_dir()? {
        if let Ok(entry) = entry {
            fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::put::data_sync(&dir, "key", b"my-data").unwrap();

        crate::rm::all_sync(&dir).unwrap();

        let new_entry = crate::get::info_sync(&dir, "key").unwrap();
        assert_eq!(new_entry, None);

        let data_exists = crate::get::hash_exists_sync(&dir, &sri);
        assert_eq!(data_exists, false);
    }

    #[test]
    fn entry() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::put::data_sync(&dir, "key", b"my-data").unwrap();

        crate::rm::entry_sync(&dir, "key").unwrap();

        let new_entry = crate::get::info_sync(&dir, "key").unwrap();
        assert_eq!(new_entry, None);

        let data_exists = crate::get::hash_exists_sync(&dir, &sri);
        assert_eq!(data_exists, true);
    }
}
