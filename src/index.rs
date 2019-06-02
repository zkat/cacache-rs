use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use digest::Digest;
use failure::Error;
use hex;
use mkdirp;
use nix::unistd::{Uid, Gid};
use serde_derive::{Deserialize, Serialize};
use serde_json::{self, json, Value};
use sha1::Sha1;
use sha2::Sha256;
use ssri::Integrity;

const INDEX_VERSION: &str = "5";

#[derive(Deserialize, Serialize)]
struct Entry {
    key: String,
    integrity: String,
    time: u128,
    size: u128,
    metadata: Value,
}

pub struct Inserter {
    cache: PathBuf,
    key: String,
    sri: Integrity,
    size: Option<u128>,
    time: Option<u128>,
    metadata: Option<Value>,
    uid: Option<Uid>,
    gid: Option<Gid>,
}

impl Inserter {
    pub fn size(mut self, size: u128) -> Self {
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

    pub fn chown(mut self, uid: Option<Uid>, gid: Option<Gid>) -> Self {
        self.uid = uid;
        self.gid = gid;
        self
    }

    pub fn commit(self) -> Result<(), Error> {
        let bucket = bucket_path(&self.cache, &self.key);
        if let Some(path) = mkdirp::mkdirp(bucket.parent().unwrap())? {
            chownr::chownr(path.as_path(), self.uid, self.gid)?;
        }
        let stringified = serde_json::to_string(&Entry {
            key: self.key.to_owned(),
            integrity: self.sri.to_string(),
            time: self.time.unwrap_or_else(now),
            size: self.size.unwrap_or(0),
            metadata: self.metadata.unwrap_or_else(|| json!(null)),
        })?;
        let str = format!("\n{}\t{}", hash_entry(&stringified), stringified);
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&bucket)?
            .write_all(&str.into_bytes())?;
        chownr::chownr(bucket.as_path(), self.uid, self.gid)?;
        Ok(())
    }
}

pub fn insert(cache: &Path, key: &str, sri: Integrity) -> Inserter {
    Inserter {
        cache: cache.to_path_buf(),
        key: String::from(key),
        size: None,
        sri,
        time: None,
        metadata: None,
        uid: None,
        gid: None,
    }
}

pub fn find(_cache: &Path, _key: &str) {
    unimplemented!();
}

pub fn delete(_cache: &Path, _key: &str) {
    unimplemented!();
}

pub fn ls(_cache: &Path) {
    unimplemented!();
}

fn bucket_path(cache: &Path, key: &str) -> PathBuf {
    let hashed = hash_key(&key);
    let mut path = PathBuf::new();
    path.push(cache);
    path.push(format!("index-v{}", INDEX_VERSION));
    path.push(&hashed[0..2]);
    path.push(&hashed[2..4]);
    path.push(&hashed[4..]);
    path
}

fn hash_key(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input(&key);
    hex::encode(hasher.result())
}

fn hash_entry(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.input(&key);
    hex::encode(hasher.result())
}

fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    #[test]
    fn insert_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        insert(&dir, "hello", sri).time(time).commit().unwrap();
        let entry = std::fs::read_to_string(bucket_path(&dir, "hello")).unwrap();
        assert_eq!(
            entry, "\n251d18a2b33264ea8655695fd23c88bd874cdea2c3dc9d8f9b7596717ad30fec\t{\"key\":\"hello\",\"integrity\":\"sha1-deadbeef\",\"time\":1234567,\"size\":0,\"metadata\":null}")
    }
}
