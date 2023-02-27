#[cfg(feature = "async-std")]
use async_std::fs as afs;
#[cfg(all(test, feature = "tokio"))]
use tokio::fs as afs;

#[cfg(all(test, feature = "async-std"))]
pub use async_std::task::block_on;

#[cfg(all(test, feature = "tokio"))]
lazy_static::lazy_static! {
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
}

#[cfg(all(test, feature = "tokio"))]
#[inline]
pub fn block_on<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    TOKIO_RUNTIME.block_on(future)
}

use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

const NUM_REPEATS: usize = 10;

fn baseline_read_sync(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("test_file");
    let data = b"hello world";
    let mut fd = File::create(&path).unwrap();
    fd.write_all(data).unwrap();
    drop(fd);
    c.bench_function("baseline_read_sync", move |b| {
        b.iter(|| fs::read(&path).unwrap())
    });
}

fn baseline_read_many_sync(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let paths: Vec<_> = (0..)
        .take(NUM_REPEATS)
        .map(|i| tmp.path().join(format!("test_file_{i}")))
        .collect();
    let data = b"hello world";
    for path in paths.iter() {
        let mut fd = File::create(path).unwrap();
        fd.write_all(data).unwrap();
        drop(fd);
    }
    c.bench_function("baseline_read_many_sync", move |b| {
        b.iter(|| {
            for path in paths.iter() {
                fs::read(black_box(&path)).unwrap();
            }
        })
    });
}

fn baseline_read_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("test_file");
    let data = b"hello world";
    let mut fd = File::create(&path).unwrap();
    fd.write_all(data).unwrap();
    drop(fd);
    c.bench_function("baseline_read_async", move |b| {
        b.iter(|| block_on(afs::read(&path)))
    });
}

fn baseline_read_many_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let paths: Vec<_> = (0..)
        .take(NUM_REPEATS)
        .map(|i| tmp.path().join(format!("test_file_{i}")))
        .collect();
    let data = b"hello world";
    for path in paths.iter() {
        let mut fd = File::create(path).unwrap();
        fd.write_all(data).unwrap();
        drop(fd);
    }
    c.bench_function("baseline_read_many_async", move |b| {
        b.iter(|| {
            let tasks = paths.iter().map(|path| afs::read(black_box(path)));
            block_on(futures::future::join_all(tasks));
        })
    });
}

fn read_hash_sync(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    let sri = cacache::write_sync(&cache, "hello", data).unwrap();
    c.bench_function("get::data_hash_sync", move |b| {
        b.iter(|| cacache::read_hash_sync(black_box(&cache), black_box(&sri)).unwrap())
    });
}

fn read_hash_many_sync(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data: Vec<_> = (0..)
        .take(NUM_REPEATS)
        .map(|i| format!("test_file_{i}"))
        .collect();
    let sris: Vec<_> = data
        .iter()
        .map(|datum| cacache::write_sync(&cache, "hello", datum).unwrap())
        .collect();
    c.bench_function("get::data_hash_many_sync", move |b| {
        b.iter(|| {
            for sri in sris.iter() {
                cacache::read_hash_sync(black_box(&cache), black_box(sri)).unwrap();
            }
        })
    });
}

fn read_sync(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    cacache::write_sync(&cache, "hello", data).unwrap();
    c.bench_function("get::data_sync", move |b| {
        b.iter(|| cacache::read_sync(black_box(&cache), black_box(String::from("hello"))).unwrap())
    });
}

fn read_hash_sync_big_data(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = vec![1; 1024 * 1024 * 5];
    let sri = cacache::write_sync(&cache, "hello", data).unwrap();
    c.bench_function("get_hash_big_data", move |b| {
        b.iter(|| cacache::read_hash_sync(black_box(&cache), black_box(&sri)).unwrap())
    });
}

fn read_hash_many_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data: Vec<_> = (0..)
        .take(NUM_REPEATS)
        .map(|i| format!("test_file_{i}"))
        .collect();
    let sris: Vec<_> = data
        .iter()
        .map(|datum| cacache::write_sync(&cache, "hello", datum).unwrap())
        .collect();
    c.bench_function("get::data_hash_many", move |b| {
        b.iter(|| {
            let tasks = sris
                .iter()
                .map(|sri| cacache::read_hash(black_box(&cache), black_box(sri)));
            block_on(futures::future::join_all(tasks));
        })
    });
}

fn read_hash_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    let sri = cacache::write_sync(&cache, "hello", data).unwrap();
    c.bench_function("get::data_hash", move |b| {
        b.iter(|| block_on(cacache::read_hash(black_box(&cache), black_box(&sri))).unwrap())
    });
}

fn read_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    cacache::write_sync(&cache, "hello", data).unwrap();
    c.bench_function("get::data", move |b| {
        b.iter(|| block_on(cacache::read(black_box(&cache), black_box("hello"))).unwrap())
    });
}

fn read_hash_async_big_data(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = vec![1; 1024 * 1024 * 5];
    let sri = cacache::write_sync(&cache, "hello", data).unwrap();
    c.bench_function("get::data_big_data", move |b| {
        b.iter(|| block_on(cacache::read_hash(black_box(&cache), black_box(&sri))).unwrap())
    });
}

fn write_hash_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    c.bench_function("put::data", move |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for i in 0..iters {
                block_on(cacache::write_hash(&cache, format!("hello world{i}"))).unwrap();
            }
            start.elapsed()
        })
    });
}

#[cfg(feature = "link_to")]
fn create_tmpfile(tmp: &tempfile::TempDir, buf: &[u8]) -> PathBuf {
    let dir = tmp.path().to_owned();
    let target = dir.join("target-file");
    std::fs::create_dir_all(target.parent().unwrap().clone()).unwrap();
    let mut file = File::create(target.clone()).unwrap();
    file.write_all(buf).unwrap();
    file.flush().unwrap();
    target
}

#[cfg(feature = "link_to")]
fn link_to_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let target = create_tmpfile(&tmp, b"hello world");

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    c.bench_function("link_to::file", move |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for i in 0..iters {
                block_on(cacache::link_to(
                    &cache,
                    format!("key{}", i),
                    target.clone(),
                ))
                .unwrap();
            }
            start.elapsed()
        })
    });
}

#[cfg(feature = "link_to")]
fn link_to_hash_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let target = create_tmpfile(&tmp, b"hello world");

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    c.bench_function("link_to::file_hash", move |b| {
        b.iter(|| block_on(cacache::link_to_hash(&cache, target.clone())).unwrap())
    });
}

#[cfg(feature = "link_to")]
fn link_to_sync(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let target = create_tmpfile(&tmp, b"hello world");

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    c.bench_function("link_to::file_sync", move |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for i in 0..iters {
                cacache::link_to_sync(&cache, format!("key{}", i), target.clone()).unwrap();
            }
            start.elapsed()
        })
    });
}

#[cfg(feature = "link_to")]
fn link_to_hash_sync(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let target = create_tmpfile(&tmp, b"hello world");

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    c.bench_function("link_to::file_hash_sync", move |b| {
        b.iter(|| cacache::link_to_hash_sync(&cache, target.clone()).unwrap())
    });
}

criterion_group!(
    benches,
    baseline_read_sync,
    baseline_read_many_sync,
    baseline_read_async,
    baseline_read_many_async,
    read_hash_async,
    read_hash_many_async,
    read_async,
    write_hash_async,
    read_hash_sync,
    read_hash_many_sync,
    read_sync,
    read_hash_async_big_data,
    read_hash_sync_big_data
);

#[cfg(feature = "link_to")]
criterion_group!(
    link_to_benches,
    link_to_async,
    link_to_hash_async,
    link_to_sync,
    link_to_hash_sync
);

#[cfg(feature = "link_to")]
criterion_main!(benches, link_to_benches);
#[cfg(not(feature = "link_to"))]
criterion_main!(benches);
