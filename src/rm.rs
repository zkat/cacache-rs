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
    use super::*;

    #[test]
    fn rm_test() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        crate::put::data(&dir, "key", b"my-data").unwrap();
        let data = crate::get::read(&dir, "key").unwrap();
        assert_eq!(data, b"my-data");
        print!("{:?}", data);
        all(&dir).unwrap();
        // panics on unwrap
        let new_data = crate::get::read(&dir, "key").unwrap_or([].to_vec());
        assert_eq!(new_data.len(), 0);
    }
}
