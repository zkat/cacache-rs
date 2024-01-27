//! Raw access to the cache index. Use with caution!

use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use digest::Digest;
use either::{Left, Right};
#[cfg(any(feature = "async-std", feature = "tokio"))]
use futures::stream::StreamExt;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use sha1::Sha1;
use sha2::Sha256;
use ssri::Integrity;
use walkdir::WalkDir;

#[cfg(any(feature = "async-std", feature = "tokio"))]
use crate::async_lib::{AsyncBufReadExt, AsyncWriteExt};
use crate::content::path::content_path;
use crate::errors::{IoErrorExt, Result};
use crate::put::WriteOpts;

const INDEX_VERSION: &str = "5";

/// Represents a cache index entry, which points to content.
#[derive(PartialEq, Debug)]
pub struct Metadata {
    /// Key this entry is stored under.
    pub key: String,
    /// Integrity hash for the stored data. Acts as a key into {cache}/content.
    pub integrity: Integrity,
    /// Timestamp in unix milliseconds when this entry was written.
    pub time: u128,
    /// Size of data associated with this entry.
    pub size: usize,
    /// Arbitrary JSON  associated with this entry.
    pub metadata: Value,
    /// Raw metadata in binary form. Can be different from JSON metadata.
    pub raw_metadata: Option<Vec<u8>>,
}

#[derive(Deserialize, Serialize, Debug)]
struct SerializableMetadata {
    key: String,
    integrity: Option<String>,
    time: u128,
    size: usize,
    metadata: Value,
    raw_metadata: Option<Vec<u8>>,
}

impl PartialEq for SerializableMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for SerializableMetadata {}

impl Hash for SerializableMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

/// Raw insertion into the cache index.
pub fn insert(cache: &Path, key: &str, opts: WriteOpts) -> Result<Integrity> {
    let bucket = bucket_path(cache, key);
    fs::create_dir_all(bucket.parent().unwrap()).with_context(|| {
        format!(
            "Failed to create index bucket directory: {:?}",
            bucket.parent().unwrap()
        )
    })?;
    let stringified = serde_json::to_string(&SerializableMetadata {
        key: key.to_owned(),
        integrity: opts.sri.clone().map(|x| x.to_string()),
        time: opts.time.unwrap_or_else(now),
        size: opts.size.unwrap_or(0),
        metadata: opts.metadata.unwrap_or(serde_json::Value::Null),
        raw_metadata: opts.raw_metadata,
    })
    .with_context(|| format!("Failed to serialize entry with key `{key}`"))?;

    let mut buck = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&bucket)
        .with_context(|| format!("Failed to create or open index bucket at {bucket:?}"))?;

    let out = format!("\n{}\t{}", hash_entry(&stringified), stringified);
    buck.write_all(out.as_bytes())
        .with_context(|| format!("Failed to write to index bucket at {bucket:?}"))?;
    buck.flush()
        .with_context(|| format!("Failed to flush bucket at {bucket:?}"))?;
    Ok(opts
        .sri
        .or_else(|| "sha1-deadbeef".parse::<Integrity>().ok())
        .unwrap())
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
/// Asynchronous raw insertion into the cache index.
pub async fn insert_async<'a>(cache: &'a Path, key: &'a str, opts: WriteOpts) -> Result<Integrity> {
    let bucket = bucket_path(cache, key);
    crate::async_lib::create_dir_all(bucket.parent().unwrap())
        .await
        .with_context(|| {
            format!(
                "Failed to create index bucket directory: {:?}",
                bucket.parent().unwrap()
            )
        })?;
    let stringified = serde_json::to_string(&SerializableMetadata {
        key: key.to_owned(),
        integrity: opts.sri.clone().map(|x| x.to_string()),
        time: opts.time.unwrap_or_else(now),
        size: opts.size.unwrap_or(0),
        metadata: opts.metadata.unwrap_or(serde_json::Value::Null),
        raw_metadata: opts.raw_metadata,
    })
    .with_context(|| format!("Failed to serialize entry with key `{key}`"))?;

    let mut buck = crate::async_lib::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&bucket)
        .await
        .with_context(|| format!("Failed to create or open index bucket at {bucket:?}"))?;

    let out = format!("\n{}\t{}", hash_entry(&stringified), stringified);
    buck.write_all(out.as_bytes())
        .await
        .with_context(|| format!("Failed to write to index bucket at {bucket:?}"))?;
    buck.flush()
        .await
        .with_context(|| format!("Failed to flush bucket at {bucket:?}"))?;
    Ok(opts
        .sri
        .or_else(|| "sha1-deadbeef".parse::<Integrity>().ok())
        .unwrap())
}

/// Raw index Metadata access.
pub fn find(cache: &Path, key: &str) -> Result<Option<Metadata>> {
    let bucket = bucket_path(cache, key);
    Ok(bucket_entries(&bucket)
        .with_context(|| format!("Failed to read index bucket entries from {bucket:?}"))?
        .into_iter()
        .fold(None, |acc, entry| {
            if entry.key == key {
                if let Some(integrity) = entry.integrity {
                    let integrity: Integrity = match integrity.parse() {
                        Ok(sri) => sri,
                        _ => return acc,
                    };
                    Some(Metadata {
                        key: entry.key,
                        integrity,
                        size: entry.size,
                        time: entry.time,
                        metadata: entry.metadata,
                        raw_metadata: entry.raw_metadata,
                    })
                } else {
                    None
                }
            } else {
                acc
            }
        }))
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
/// Asynchronous raw index Metadata access.
pub async fn find_async(cache: &Path, key: &str) -> Result<Option<Metadata>> {
    let bucket = bucket_path(cache, key);
    Ok(bucket_entries_async(&bucket)
        .await
        .with_context(|| format!("Failed to read index bucket entries from {bucket:?}"))?
        .into_iter()
        .fold(None, |acc, entry| {
            if entry.key == key {
                if let Some(integrity) = entry.integrity {
                    let integrity: Integrity = match integrity.parse() {
                        Ok(sri) => sri,
                        _ => return acc,
                    };
                    Some(Metadata {
                        key: entry.key,
                        integrity,
                        size: entry.size,
                        time: entry.time,
                        metadata: entry.metadata,
                        raw_metadata: entry.raw_metadata,
                    })
                } else {
                    None
                }
            } else {
                acc
            }
        }))
}

/// Deletes an index entry, without deleting the actual cache data entry.
pub fn delete(cache: &Path, key: &str) -> Result<()> {
    insert(
        cache,
        key,
        WriteOpts {
            algorithm: None,
            size: None,
            sri: None,
            time: None,
            metadata: None,
            raw_metadata: None,
        },
    )
    .map(|_| ())
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
/// Asynchronously deletes an index entry, without deleting the actual cache
/// data entry.
pub async fn delete_async(cache: &Path, key: &str) -> Result<()> {
    insert(
        cache,
        key,
        WriteOpts {
            algorithm: None,
            size: None,
            sri: None,
            time: None,
            metadata: None,
            raw_metadata: None,
        },
    )
    .map(|_| ())
}

/// Lists raw index Metadata entries.
pub fn ls(cache: &Path) -> impl Iterator<Item = Result<Metadata>> {
    let cache_path = cache.join(format!("index-v{INDEX_VERSION}"));
    let cloned = cache_path.clone();
    WalkDir::new(&cache_path)
        .into_iter()
        .map(move |bucket| {
            let bucket = bucket
                .map_err(|e| match e.io_error() {
                    Some(io_err) => std::io::Error::new(io_err.kind(), io_err.kind().to_string()),
                    None => crate::errors::io_error("Unexpected error"),
                })
                .with_context(|| {
                    format!(
                        "Error while walking cache index directory at {}",
                        cloned.display()
                    )
                })?;

            if bucket.file_type().is_dir() {
                return Ok(Vec::new());
            }

            let owned_path = bucket.path().to_owned();
            Ok(bucket_entries(bucket.path())
                .with_context(|| {
                    format!("Error getting bucket entries from {}", owned_path.display())
                })?
                .into_iter()
                .rev()
                .collect::<HashSet<SerializableMetadata>>()
                .into_iter()
                .filter_map(|se| {
                    if let Some(i) = se.integrity {
                        Some(Metadata {
                            key: se.key,
                            integrity: i.parse().unwrap(),
                            time: se.time,
                            size: se.size,
                            metadata: se.metadata,
                            raw_metadata: se.raw_metadata,
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
    let hashed = hash_key(key);
    cache
        .join(format!("index-v{INDEX_VERSION}"))
        .join(&hashed[0..2])
        .join(&hashed[2..4])
        .join(&hashed[4..])
}

fn hash_key(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key);
    hex::encode(hasher.finalize())
}

fn hash_entry(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key);
    hex::encode(hasher.finalize())
}

fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn bucket_entries(bucket: &Path) -> std::io::Result<Vec<SerializableMetadata>> {
    use std::io::{BufRead, BufReader};
    fs::File::open(bucket)
        .map(|file| {
            BufReader::new(file)
                .lines()
                .map_while(std::result::Result::ok)
                .filter_map(|entry| {
                    let entry_str = match entry.split('\t').collect::<Vec<&str>>()[..] {
                        [hash, entry_str] if hash_entry(entry_str) == hash => entry_str,
                        // Something's wrong with the entry. Abort.
                        _ => return None,
                    };
                    serde_json::from_str::<SerializableMetadata>(entry_str).ok()
                })
                .collect()
        })
        .or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                Ok(Vec::new())
            } else {
                Err(err)?
            }
        })
}

#[cfg(any(feature = "async-std", feature = "tokio"))]
async fn bucket_entries_async(bucket: &Path) -> std::io::Result<Vec<SerializableMetadata>> {
    let file_result = crate::async_lib::File::open(bucket).await;
    let file = if let Err(err) = file_result {
        if err.kind() == ErrorKind::NotFound {
            return Ok(Vec::new());
        }
        return Err(err)?;
    } else {
        file_result.unwrap()
    };
    let mut vec = Vec::new();
    let mut lines =
        crate::async_lib::lines_to_stream(crate::async_lib::BufReader::new(file).lines());
    while let Some(line) = lines.next().await {
        if let Ok(entry) = line {
            let entry_str = match entry.split('\t').collect::<Vec<&str>>()[..] {
                [hash, entry_str] if hash_entry(entry_str) == hash => entry_str,
                // Something's wrong with the entry. Abort.
                _ => continue,
            };
            if let Ok(serialized) = serde_json::from_str::<SerializableMetadata>(entry_str) {
                vec.push(serialized);
            }
        }
    }
    Ok(vec)
}

/// Builder for options and flags for remove cache entry.
#[derive(Clone, Default)]
pub struct RemoveOpts {
    pub(crate) remove_fully: bool,
}

impl RemoveOpts {
    /// Creates cache remove options.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the remove fully option
    /// If remove_fully is set to true then the index file itself will be physically deleted rather than appending a null.
    pub fn remove_fully(mut self, remove_fully: bool) -> Self {
        self.remove_fully = remove_fully;
        self
    }

    /// Removes an individual index metadata entry. The associated content will be left in the cache.
    pub fn remove_sync<P, K>(self, cache: P, key: K) -> Result<()>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
    {
        if !self.remove_fully {
            delete(cache.as_ref(), key.as_ref())
        } else {
            if let Some(meta) = crate::metadata_sync(cache.as_ref(), key.as_ref())? {
                let content = content_path(cache.as_ref(), &meta.integrity);
                fs::remove_file(&content)
                    .with_context(|| format!("Failed to remove content at {content:?}"))?;
            }
            let bucket = bucket_path(cache.as_ref(), key.as_ref());
            fs::remove_file(&bucket)
                .with_context(|| format!("Failed to remove bucket at {bucket:?}"))
        }
    }

    /// Removes an individual index metadata entry. The associated content will be left in the cache.
    pub async fn remove<P, K>(self, cache: P, key: K) -> Result<()>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
    {
        if !self.remove_fully {
            delete_async(cache.as_ref(), key.as_ref()).await
        } else {
            if let Some(meta) = crate::metadata(cache.as_ref(), key.as_ref()).await? {
                let content = content_path(cache.as_ref(), &meta.integrity);
                crate::async_lib::remove_file(&content)
                    .await
                    .with_context(|| format!("Failed to remove content at {content:?}"))?;
            }
            let bucket = bucket_path(cache.as_ref(), key.as_ref());
            crate::async_lib::remove_file(&bucket)
                .await
                .with_context(|| format!("Failed to remove bucket at {bucket:?}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[cfg(feature = "async-std")]
    use async_attributes::test as async_test;
    #[cfg(feature = "tokio")]
    use tokio::test as async_test;

    const MOCK_ENTRY: &str = "\n9cbbfe2553e7c7e1773f53f0f643fdd72008faa38da53ebcb055e5e20321ae47\t{\"key\":\"hello\",\"integrity\":\"sha1-deadbeef\",\"time\":1234567,\"size\":0,\"metadata\":null,\"raw_metadata\":null}";

    fn ls_entries(dir: &Path) -> Vec<String> {
        let mut entries = ls(dir)
            .map(|x| Ok(x?.key))
            .collect::<Result<Vec<_>>>()
            .unwrap();
        entries.sort();
        entries
    }

    #[test]
    fn insert_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = WriteOpts::new().integrity(sri).time(time);
        insert(&dir, "hello", opts).unwrap();
        let entry = std::fs::read_to_string(bucket_path(&dir, "hello")).unwrap();
        assert_eq!(entry, MOCK_ENTRY);
    }

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    #[async_test]
    async fn insert_async_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = WriteOpts::new().integrity(sri).time(time);
        futures::executor::block_on(async {
            insert_async(&dir, "hello", opts).await.unwrap();
        });
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
        fs::create_dir_all(bucket.parent().unwrap()).unwrap();
        fs::write(bucket, MOCK_ENTRY).unwrap();
        let entry = find(&dir, "hello").unwrap().unwrap();
        assert_eq!(
            entry,
            Metadata {
                key: String::from("hello"),
                integrity: sri,
                time,
                size: 0,
                metadata: json!(null),
                raw_metadata: None,
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
        let opts = WriteOpts::new().integrity(sri).time(time);
        insert(&dir, "hello", opts).unwrap();
        delete(&dir, "hello").unwrap();
        assert_eq!(find(&dir, "hello").unwrap(), None);
    }

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    #[async_test]
    async fn delete_async_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = WriteOpts::new().integrity(sri).time(time);
        insert(&dir, "hello", opts).unwrap();
        futures::executor::block_on(async {
            delete_async(&dir, "hello").await.unwrap();
        });
        assert_eq!(find(&dir, "hello").unwrap(), None);
    }

    #[test]
    fn delete_fully() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let content = content_path(&dir, &"sha1-deadbeef".parse().unwrap());
        fs::create_dir_all(content.parent().unwrap()).unwrap();
        fs::write(content.as_path(), "hello").unwrap();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        insert(&dir, "hello", WriteOpts::new().integrity(sri).time(time)).unwrap();
        RemoveOpts::new()
            .remove_fully(true)
            .remove_sync(&dir, "hello")
            .unwrap();
        assert_eq!(find(&dir, "hello").unwrap(), None);
        assert!(!content.exists());
    }

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    #[async_test]
    async fn delete_fully_async() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let content = content_path(&dir, &"sha1-deadbeef".parse().unwrap());
        fs::create_dir_all(content.parent().unwrap()).unwrap();
        fs::write(content.as_path(), "hello").unwrap();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        insert(&dir, "hello", WriteOpts::new().integrity(sri).time(time)).unwrap();
        RemoveOpts::new()
            .remove_fully(true)
            .remove(&dir, "hello")
            .await
            .unwrap();
        assert_eq!(find(&dir, "hello").unwrap(), None);
        assert!(!content.exists());
    }

    #[test]
    fn round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = WriteOpts::new().integrity(sri.clone()).time(time);
        insert(&dir, "hello", opts).unwrap();
        let entry = find(&dir, "hello").unwrap().unwrap();
        assert_eq!(
            entry,
            Metadata {
                key: String::from("hello"),
                integrity: sri,
                time,
                size: 0,
                metadata: json!(null),
                raw_metadata: None,
            }
        );
    }

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    #[async_test]
    async fn round_trip_async() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = WriteOpts::new().integrity(sri.clone()).time(time);
        futures::executor::block_on(async {
            insert_async(&dir, "hello", opts).await.unwrap();
        });
        let entry = futures::executor::block_on(async {
            find_async(&dir, "hello").await.unwrap().unwrap()
        });
        assert_eq!(
            entry,
            Metadata {
                key: String::from("hello"),
                integrity: sri,
                time,
                size: 0,
                metadata: json!(null),
                raw_metadata: None,
            }
        );
    }

    #[test]
    fn ls_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = WriteOpts::new().integrity(sri.clone()).time(time);
        insert(&dir, "hello", opts).unwrap();
        let opts = WriteOpts::new().integrity(sri).time(time);
        insert(&dir, "world", opts).unwrap();

        let entries = ls_entries(&dir);
        assert_eq!(entries, vec![String::from("hello"), String::from("world")])
    }

    #[test]
    fn ls_basic_with_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri: Integrity = "sha1-deadbeef".parse().unwrap();
        let time = 1_234_567;
        let opts = WriteOpts::new().integrity(sri.clone()).time(time);
        insert(&dir, "hello", opts).unwrap();
        let opts = WriteOpts::new().integrity(sri).time(time);
        insert(&dir, "world", opts).unwrap();

        let entries = ls_entries(&dir);
        assert_eq!(entries, vec![String::from("hello"), String::from("world")]);

        delete(&dir, "hello").unwrap();
        let entries = ls_entries(&dir);
        assert_eq!(entries, vec![String::from("world")])
    }
}
