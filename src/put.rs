//! Functions for writing to cache.
use std::path::{Path, PathBuf};

use nix::unistd::{Uid, Gid};
use serde_json::Value;
use ssri::Integrity;

use crate::content::write;
use crate::index;
use crate::errors::Error;

pub fn data(cache: &Path, key: String, data: Vec<u8>) -> Result<Integrity, Error> {
    let sri = write::write(&cache, &data)?;
    Writer::new(cache, &key).integrity(sri).commit(data)
}

pub struct Writer {
    pub cache: PathBuf,
    pub key: String,
    pub sri: Option<Integrity>,
    pub size: Option<usize>,
    pub time: Option<u128>,
    pub metadata: Option<Value>,
    pub uid: Option<Uid>,
    pub gid: Option<Gid>,
}

impl Writer {
    pub fn new(cache: &Path, key: &str) -> Writer {
        Writer {
            cache: cache.to_path_buf(),
            key: String::from(key),
            sri: None,
            size: None,
            time: None,
            metadata: None,
            uid: None,
            gid: None
        }
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

    pub fn commit(self, data: Vec<u8>) -> Result<Integrity, Error> {
        if let Some(sri) = &self.sri {
            if sri.clone().check(&data).is_none() {
                return Err(Error::IntegrityError);
            }
        }
        if let Some(size) = self.size {
            if size != data.len() {
                return Err(Error::SizeError);
            }
        }
        let sri = write::write(&self.cache, &data)?;
        index::insert(self)?;
        Ok(sri)
    }
}
