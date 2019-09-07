// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

use cacache;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile;

fn get(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    let sri = cacache::put::data(&cache, "hello", data).unwrap();
    c.bench_function("read_hash", move |b| {
        b.iter(|| cacache::get::read_hash(black_box(&cache), black_box(&sri)))
    });
}

criterion_group!(benches, get);
criterion_main!(benches);
