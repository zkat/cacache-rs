use async_std::task;
use cacache;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile;

fn get_data_hash_sync(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    let sri = cacache::put::data_sync(&cache, "hello", data).unwrap();
    c.bench_function("get::data_hash_sync", move |b| {
        b.iter(|| cacache::get::data_hash_sync(black_box(&cache), black_box(&sri)).unwrap())
    });
}

fn get_data_sync(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    cacache::put::data_sync(&cache, "hello", data).unwrap();
    cacache::get::data_sync(&cache, "hello").unwrap();
    c.bench_function("get::data_sync", move |b| {
        b.iter(|| {
            cacache::get::data_sync(black_box(&cache), black_box(String::from("hello"))).unwrap()
        })
    });
}

fn get_data_hash_sync_big_data(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = vec![1; 1024 * 1024 * 5];
    let sri = cacache::put::data_sync(&cache, "hello", data).unwrap();
    c.bench_function("get_hash_big_data", move |b| {
        b.iter(|| cacache::get::data_hash_sync(black_box(&cache), black_box(&sri)).unwrap())
    });
}

fn get_data_hash_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    let sri = cacache::put::data_sync(&cache, "hello", data).unwrap();
    c.bench_function("get::data_hash", move |b| {
        b.iter(|| {
            task::block_on(cacache::get::data_hash(black_box(&cache), black_box(&sri))).unwrap()
        })
    });
}

fn get_data_async(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    cacache::put::data_sync(&cache, "hello", data).unwrap();
    c.bench_function("get::data", move |b| {
        b.iter(|| {
            task::block_on(cacache::get::data(black_box(&cache), black_box("hello"))).unwrap()
        })
    });
}

fn get_data_hash_async_big_data(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = vec![1; 1024 * 1024 * 5];
    let sri = cacache::put::data_sync(&cache, "hello", data).unwrap();
    c.bench_function("get::data_big_data", move |b| {
        b.iter(|| {
            task::block_on(cacache::get::data_hash(black_box(&cache), black_box(&sri))).unwrap()
        })
    });
}

criterion_group!(
    benches,
    get_data_hash_async,
    get_data_hash_sync,
    get_data_async,
    get_data_sync,
    get_data_hash_async_big_data,
    get_data_hash_sync_big_data
);
criterion_main!(benches);
