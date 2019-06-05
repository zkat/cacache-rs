use std::fs;
use std::path::Path;

use ssri::Integrity;

use crate::content::rm;
use crate::errors::Error;
use crate::index;

pub fn entry(cache: &Path, key: &str) -> Result<(), Error> {
    index::delete(&cache, &key)
}

pub fn content(cache: &Path, sri: &Integrity) -> Result<(), Error> {
    rm::rm(&cache, &sri)
}

pub fn all(cache: &Path) -> Result<(), Error> {
    for entry in cache.read_dir()? {
        if let Ok(entry) = entry {
            fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}
