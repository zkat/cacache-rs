use async_std::task;
use cacache;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile;

fn get_hash(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    let sri = cacache::put::data(&cache, "hello", data).unwrap();
    c.bench_function("get_hash", move |b| {
        b.iter(|| cacache::get::data_hash(black_box(&cache), black_box(&sri)).unwrap())
    });
}

fn get(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    cacache::put::data(&cache, "hello", data).unwrap();
    cacache::get::data(&cache, "hello").unwrap();
    c.bench_function("get", move |b| {
        b.iter(|| cacache::get::data(black_box(&cache), black_box(String::from("hello"))).unwrap())
    });
}

fn get_hash_big_data(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = vec![1; 1024 * 1024 * 5];
    let sri = cacache::put::data(&cache, "hello", data).unwrap();
    c.bench_function("get_hash_big_data", move |b| {
        b.iter(|| cacache::get::data_hash(black_box(&cache), black_box(&sri)).unwrap())
    });
}

fn async_get_hash(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    let sri = cacache::put::data(&cache, "hello", data).unwrap();
    c.bench_function("async_get_hash", move |b| {
        b.iter(|| {
            task::block_on(cacache::async_get::data_hash(
                black_box(&cache),
                black_box(&sri),
            ))
            .unwrap()
        })
    });
}

fn async_get(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    cacache::put::data(&cache, "hello", data).unwrap();
    c.bench_function("async_get", move |b| {
        b.iter(|| {
            task::block_on(cacache::async_get::data(
                black_box(&cache),
                black_box("hello"),
            ))
            .unwrap()
        })
    });
}

fn async_get_hash_big_data(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = vec![1; 1024 * 1024 * 5];
    let sri = cacache::put::data(&cache, "hello", data).unwrap();
    c.bench_function("async_get_hash_big_data", move |b| {
        b.iter(|| {
            task::block_on(cacache::async_get::data_hash(
                black_box(&cache),
                black_box(&sri),
            ))
            .unwrap()
        })
    });
}

criterion_group!(
    benches,
    get_hash,
    get,
    async_get_hash,
    async_get,
    get_hash_big_data,
    async_get_hash_big_data,
);
criterion_main!(benches);
