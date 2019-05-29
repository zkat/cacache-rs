use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use digest::Digest;
use hex;
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

pub fn insert(cache: &Path, key: &str, sri: Integrity) -> std::io::Result<()> {
    let bucket = bucket_path(&cache, &key);
    fs::create_dir_all(bucket.parent().unwrap())?;
    let stringified = serde_json::to_string(&Entry {
        key: String::from(key),
        integrity: sri.to_string(),
        time: now(),
        size: 0, // TODO - probably do something about this.
        metadata: json!(null),
    })
    .expect("Failed to serialize entry.");
    let str = format!("\n{}\t{}", hash_entry(&stringified), stringified);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(bucket)?
        .write_all(&str.into_bytes())
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
