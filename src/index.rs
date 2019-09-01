use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use chownr;
use digest::Digest;
use either::{Left, Right};
use hex;
use mkdirp;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha1::Sha1;
use sha2::Sha256;
use ssri::Integrity;
use walkdir::WalkDir;

use crate::errors::Error;
use crate::put::PutOpts;

const INDEX_VERSION: &str = "5";

/// Represents a cache index entry, which points to content.
#[derive(PartialEq, Debug)]
pub struct Entry {
    /// Key this entry is stored under.
    pub key: String,
    /// Integrity hash for the stored data. Acts as a key into {cache}/content.
    pub integrity: Integrity,
    /// Timestamp in unix milliseconds when this entry was written.
    pub time: u128,
    /// Size of data associated with this entry.
    pub size: usize,
    /// Arbitrary JSON metadata associated with this entry.
    pub metadata: Value,
}

#[derive(Deserialize, Serialize, Debug)]
struct SerializableEntry {
    key: String,
    integrity: Option<String>,
    time: u128,
    size: usize,
    metadata: Value,
}

impl PartialEq for SerializableEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for SerializableEntry {}

impl Hash for SerializableEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

pub fn insert(cache: &Path, key: &str, opts: PutOpts) -> Result<Integrity, Error> {
    let bucket = bucket_path(&cache, &key);
    #[cfg(unix)]
    {
        if let Some(path) = mkdirp::mkdirp(bucket.parent().unwrap())? {
            chownr::chownr(&path, opts.uid, opts.gid)?;
        }
    }
    #[cfg(windows)]
    mkdirp::mkdirp(bucket.parent().unwrap())?;
    let stringified = serde_json::to_string(&SerializableEntry {
        key: key.to_owned(),
        integrity: opts.sri.clone().map(|x| x.to_string()),
        time: opts.time.unwrap_or_else(now),
        size: opts.size.unwrap_or(0),
        metadata: opts.metadata.unwrap_or_else(|| json!(null)),
    })?;

    let mut buck = OpenOptions::new().create(true).append(true).open(&bucket)?;

    write!(buck, "\n{}\t{}", hash_entry(&stringified), stringified)?;
    #[cfg(unix)]
    chownr::chownr(&bucket, opts.uid, opts.gid)?;
    Ok(opts
        .sri
        .or_else(|| "sha1-deadbeef".parse::<Integrity>().ok())
        .unwrap())
}

pub fn find(cache: &Path, key: &str) -> Result<Option<Entry>, Error> {
    let bucket = bucket_path(cache, &key);
    Ok(bucket_entries(&bucket)?
        .into_iter()
        .fold(None, |acc, entry| {
            if entry.key == key {
                if let Some(integrity) = entry.integrity {
                    let integrity: Integrity = match integrity.parse() {
                        Ok(sri) => sri,
                        _ => return acc,
                    };
                    Some(Entry {
                        key: entry.key,
                        integrity,
                        size: entry.size,
                        time: entry.time,
                        metadata: entry.metadata,
                    })
                } else {
                    None
                }
            } else {
                acc
            }
        }))
}

pub fn delete(cache: &Path, key: &str) -> Result<(), Error> {
    insert(
        cache,
        key,
        PutOpts {
            algorithm: None,
            size: None,
            sri: None,
            time: None,
            metadata: None,
            #[cfg(unix)]
            uid: None,
            #[cfg(unix)]
            gid: None,
        },
    )
    .map(|_| ())
}

pub fn ls(cache: &Path) -> impl Iterator<Item = Result<Entry, Error>> {
    WalkDir::new(cache.join(format!("index-v{}", INDEX_VERSION)))
        .into_iter()
        .map(|bucket| {
            let bucket = bucket?;
            if bucket.file_type().is_dir() {
                return Ok(Vec::new());
            }

            Ok(bucket_entries(bucket.path())?
                .into_iter()
                .collect::<HashSet<SerializableEntry>>()
                .into_iter()
                .filter_map(|se| {
                    if let Some(i) = se.integrity {
                        Some(Entry {
                            key: se.key,
                            integrity: i.parse().unwrap(),
                            time: se.time,
                            size: se.size,
                            metadata: se.metadata,
                        })
                    } else {
                        None
                    }
                })
                .collect())
        })
        .flat_map(|res| match res {
            Ok(it) => Left(it.into_iter().map(Ok)),
            Err(err) => Right(std::iter::once(Err(err))),
        })
}

fn bucket_path(cache: &Path, key: &str) -> PathBuf {
    let hashed = hash_key(&key);
    cache
        .join(format!("index-v{}", INDEX_VERSION))
        .join(&hashed[0..2])
        .join(&hashed[2..4])
        .join(&hashed[4..])
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

fn bucket_entries(bucket: &Path) -> Result<Vec<SerializableEntry>, Error> {
    use std::io::{BufRead, BufReader};
    fs::File::open(bucket)
        .map(|file| {
            BufReader::new(file)
                .lines()
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let entry_str = match entry.split('\t').collect::<Vec<&str>>()[..] {
                        [hash, entry_str] if hash_entry(entry_str) == hash => entry_str,
                        // Something's wrong with the entry. Abort.
                        _ => return None,
                    };
                    serde_json::from_str::<SerializableEntry>(entry_str).ok()
                })
                .collect()
        })
        .or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                Ok(Vec::new())
            } else {
                Err(err.into())
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    const MOCK_ENTRY: &str = "\n251d18a2b33264ea8655695fd23c88bd874cdea2c3dc9d8f9b7596717ad30fec\t{\"key\":\"hello\",\"integrity\":\"sha1-deadbeef\",\"time\":1234567,\"size\":0,\"metadata\":null}";

    #[test]
    fn insert_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = PutOpts::new().integrity(sri).time(time);
        insert(&dir, "hello", opts).unwrap();
        let entry = std::fs::read_to_string(bucket_path(&dir, "hello")).unwrap();
        assert_eq!(entry, MOCK_ENTRY);
    }

    #[test]
    fn find_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let bucket = bucket_path(&dir, "hello");
        mkdirp::mkdirp(bucket.parent().unwrap()).unwrap();
        fs::write(bucket, MOCK_ENTRY).unwrap();
        let entry = find(&dir, "hello").unwrap().unwrap();
        assert_eq!(
            entry,
            Entry {
                key: String::from("hello"),
                integrity: sri,
                time,
                size: 0,
                metadata: json!(null)
            }
        );
    }

    #[test]
    fn find_none() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        assert_eq!(find(&dir, "hello").unwrap(), None);
    }

    #[test]
    fn delete_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = PutOpts::new().integrity(sri).time(time);
        insert(&dir, "hello", opts).unwrap();
        delete(&dir, "hello").unwrap();
        assert_eq!(find(&dir, "hello").unwrap(), None);
    }

    #[test]
    fn ls_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = PutOpts::new().integrity(sri.clone()).time(time);
        insert(&dir, "hello", opts).unwrap();
        let opts = PutOpts::new().integrity(sri).time(time);
        insert(&dir, "world", opts).unwrap();

        let mut entries = ls(&dir)
            .map(|x| Ok(x?.key))
            .collect::<Result<Vec<_>, Error>>()
            .unwrap();
        entries.sort();
        assert_eq!(entries, vec![String::from("hello"), String::from("world")])
    }
}
