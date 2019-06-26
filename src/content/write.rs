use std::fs::DirBuilder;
use std::io::prelude::*;
use std::path::Path;

use atomicwrites::{AtomicFile, AllowOverwrite};
use ssri::Integrity;

use crate::content::path;
use crate::errors::Error;

pub fn write(cache: &Path, data: &[u8]) -> Result<Integrity, Error> {
    let sri = Integrity::from(&data);
    let cpath = path::content_path(&cache, &sri);
    DirBuilder::new().recursive(true).create(cpath.parent().unwrap())?;
    let file = AtomicFile::new(&cpath, AllowOverwrite);
    file.write(|f| f.write_all(&data))?;
    Ok(sri)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    #[test]
    fn basic_write() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = write(&dir, b"hello world").unwrap();
        assert_eq!(
            sri.to_string(),
            Integrity::from(b"hello world").to_string()
        );
        assert_eq!(
            std::fs::read(path::content_path(&dir, &sri)).unwrap(),
            b"hello world"
        );
    }
}
