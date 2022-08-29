use std::sync::Mutex;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng, thread_rng};
use rust_linked_list::mine::NaiveLruCache;
use concurrent_lru::sharded::LruCache;
use rayon::prelude::*;
use rayon::ThreadPool;

const OP_COUNT:usize = 1000000;

fn criterion_benchmark(c: &mut Criterion) {
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .unwrap();

    c.bench_function("my_cache", |b| b.iter(|| {
        let mut cache: NaiveLruCache<i32> = NaiveLruCache::new(256);
        (0..OP_COUNT).for_each(|_|{
            let item = thread_rng().gen_range(0..200);
            cache.put(item);
        })
    }));

    c.bench_function("concurrent_lru", |b| b.iter(|| {
        let mut cache: LruCache<i32, i32> = LruCache::new(16);
        thread_pool.install(||{
            (0..OP_COUNT).into_par_iter().for_each(|_|{
                let item = thread_rng().gen_range(0..200);
                cache.get_or_init(item, 1, |x|1);
            })
        });
    }));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
