use std::fs::{self, File};
use std::path::Path;

use ssri::{Algorithm, Integrity, IntegrityChecker};

use crate::content::path;
use crate::errors::Error;

pub struct Reader {
    fd: File,
    checker: IntegrityChecker,
}

impl std::io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let amt = self.fd.read(buf)?;
        self.checker.input(&buf);
        Ok(amt)
    }
}

impl Reader {
    pub fn check(self) -> Result<Algorithm, Error> {
        self.checker.result().ok_or(Error::IntegrityError)
    }
}

pub fn open<'a>(cache: &Path, sri: Integrity) -> Result<Reader, Error> {
    Ok(Reader {
        fd: File::open(cache)?,
        checker: IntegrityChecker::new(sri),
    })
}

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
