use std::fs;
use std::path::Path;

use ssri::Integrity;

use crate::content::path;
use crate::errors::Error;

pub fn read(cache: &Path, sri: &Integrity) -> Result<Vec<u8>, Error> {
    let cpath = path::content_path(&cache, &sri);
    let ret = fs::read(&cpath)?;
    if sri.check(&ret).is_some() {
        Ok(ret)
    } else {
        Err(Error::IntegrityError)
    }
}

pub fn copy(cache: &Path, sri: &Integrity, to: &Path) -> Result<u64, Error> {
    let cpath = path::content_path(&cache, &sri);
    let ret = fs::copy(&cpath, to)?;
    let data = fs::read(cpath)?;
    if sri.check(data).is_some() {
        Ok(ret)
    } else {
        Err(Error::IntegrityError)
    }
}

pub fn has_content(cache: &Path, sri: &Integrity) -> Option<Integrity> {
    if path::content_path(&cache, &sri).exists() {
        Some(sri.clone())
    } else {
        None
    }
}
