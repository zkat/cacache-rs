use atomicwrites::{AtomicFile, AllowOverwrite};
use crate::content::path;
use ssri::{Algorithm, Integrity};
use std::fs::DirBuilder;
use std::io::{self, prelude::*};
use std::path::Path;

pub fn write(cache: &Path, data: &[u8]) -> io::Result<Integrity> {
    let sri = Integrity::from(&data, Algorithm::Sha256);
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
        println!("dir: {}", dir.to_str().unwrap());
        let sri = write(&dir, b"hello world").unwrap();
        assert_eq!(
            sri.to_string(),
            Integrity::from(b"hello world", Algorithm::Sha256).to_string()
        );
        assert_eq!(
            std::fs::read(path::content_path(&dir, &sri)).unwrap(),
            b"hello world"
        );
    }
}
