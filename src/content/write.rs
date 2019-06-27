use std::fs::DirBuilder;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use ssri::{Algorithm, Integrity, IntegrityOpts};
use tempfile::NamedTempFile;

use crate::content::path;
use crate::errors::Error;

pub struct Writer {
    cache: PathBuf,
    builder: IntegrityOpts,
    tmpfile: NamedTempFile,
}

impl Writer {
    pub fn new(cache: &Path, algo: Algorithm) -> Result<Writer, Error> {
        let cache_path = cache.to_path_buf();
        let mut tmp_path = cache_path.clone();
        tmp_path.push("tmp");
        DirBuilder::new().recursive(true).create(&tmp_path)?;
        Ok(Writer {
            cache: cache_path,
            builder: IntegrityOpts::new().algorithm(algo),
            tmpfile: NamedTempFile::new_in(tmp_path)?,
        })
    }

    pub fn close(self) -> Result<Integrity, Error> {
        let sri = self.builder.result();
        let cpath = path::content_path(&self.cache, &sri);
        DirBuilder::new()
            .recursive(true)
            .create(cpath.parent().unwrap())?;
        self.tmpfile.persist(cpath)?;
        Ok(sri)
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.builder.input(&buf);
        self.tmpfile.write(&buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.tmpfile.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    #[test]
    fn basic_write() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut writer = Writer::new(&dir, Algorithm::Sha256).unwrap();
        writer.write_all(b"hello world").unwrap();
        let sri = writer.close().unwrap();
        assert_eq!(sri.to_string(), Integrity::from(b"hello world").to_string());
        assert_eq!(
            std::fs::read(path::content_path(&dir, &sri)).unwrap(),
            b"hello world"
        );
    }
}
