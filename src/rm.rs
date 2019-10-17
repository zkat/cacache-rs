//! Functions for removing things from the cache.
use std::fs;
use std::path::Path;

use ssri::Integrity;

use crate::content::rm;
use crate::errors::Error;
use crate::index;

/// Removes an individual index entry. The associated content will be left
/// intact.
pub fn entry<P: AsRef<Path>>(cache: P, key: &str) -> Result<(), Error> {
    index::delete(cache.as_ref(), &key)
}

/// Removes an individual content entry. Any index entries pointing to this
/// content will become invalidated.
pub fn content<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<(), Error> {
    rm::rm(cache.as_ref(), &sri)
}

/// Removes entire contents of the cache, including temporary files, the entry
/// index, and all content data.
pub fn all<P: AsRef<Path>>(cache: P) -> Result<(), Error> {
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
    fn all() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::put::data(&dir, "key", b"my-data").unwrap();

        crate::rm::all(&dir).unwrap();

        let new_entry = crate::get::info(&dir, "key").unwrap();
        assert_eq!(new_entry, None);

        let data_exists = crate::get::hash_exists(&dir, &sri);
        assert_eq!(data_exists, false);
    }

    #[test]
    fn entry() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::put::data(&dir, "key", b"my-data").unwrap();

        crate::rm::entry(&dir, "key").unwrap();

        let new_entry = crate::get::info(&dir, "key").unwrap();
        assert_eq!(new_entry, None);

        let data_exists = crate::get::hash_exists(&dir, &sri);
        assert_eq!(data_exists, true);
    }
}
