//! Functions for writing to cache.
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use nix::unistd::{Gid, Uid};
use serde_json::Value;
use ssri::{Algorithm, Integrity};

use crate::content::write;
use crate::errors::Error;
use crate::index;

pub fn data<P, D, K>(cache: P, key: K, data: D) -> Result<Integrity, Error>
where
    P: AsRef<Path>,
    D: AsRef<[u8]>,
    K: AsRef<str>,
{
    let mut writer = PutOpts::new()
        .algorithm(Algorithm::Sha256)
        .open(cache.as_ref(), key.as_ref())?;
    writer.write_all(data.as_ref())?;
    writer.commit()
}

#[derive(Clone, Default)]
pub struct PutOpts {
    pub algorithm: Option<Algorithm>,
    pub sri: Option<Integrity>,
    pub size: Option<usize>,
    pub time: Option<u128>,
    pub metadata: Option<Value>,
    pub uid: Option<Uid>,
    pub gid: Option<Gid>,
}

impl PutOpts {
    pub fn new() -> PutOpts {
        Default::default()
    }

    pub fn open<P, K>(self, cache: P, key: K) -> Result<Put, Error>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
    {
        Ok(Put {
            cache: cache.as_ref().to_path_buf(),
            key: String::from(key.as_ref()),
            written: 0,
            writer: write::Writer::new(
                cache.as_ref(),
                *self.algorithm.as_ref().unwrap_or(&Algorithm::Sha256),
            )?,
            opts: self,
        })
    }

    pub fn algorithm(mut self, algo: Algorithm) -> Self {
        self.algorithm = Some(algo);
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn time(mut self, time: u128) -> Self {
        self.time = Some(time);
        self
    }

    pub fn integrity(mut self, sri: Integrity) -> Self {
        self.sri = Some(sri);
        self
    }

    pub fn chown(mut self, uid: Option<Uid>, gid: Option<Gid>) -> Self {
        self.uid = uid;
        self.gid = gid;
        self
    }
}

pub struct Put {
    pub cache: PathBuf,
    pub key: String,
    pub written: usize,
    pub(crate) writer: write::Writer,
    pub opts: PutOpts,
}

impl Write for Put {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl Put {
    pub fn commit(self) -> Result<Integrity, Error> {
        let writer_sri = self.writer.close()?;
        if let Some(sri) = &self.opts.sri {
            // TODO - ssri should have a .matches method
            let algo = sri.pick_algorithm();
            let matched = sri
                .hashes
                .iter()
                .take_while(|h| h.algorithm == algo)
                .find(|&h| *h == writer_sri.hashes[0]);
            if matched.is_none() {
                return Err(Error::IntegrityError);
            }
        }
        if let Some(size) = self.opts.size {
            if size != self.written {
                return Err(Error::SizeError);
            }
        }
        index::insert(&self.cache, &self.key, self.opts)?;
        Ok(writer_sri)
    }
}
