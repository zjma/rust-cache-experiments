use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::thread::sleep;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng, thread_rng};
use rust_linked_list::mine::NaiveLruCache;
use concurrent_lru::sharded::LruCache as GithubCache1;
use rayon::prelude::*;
use rayon::{spawn, ThreadPool};
use lru::LruCache as GithubCache2;
use rust_linked_list::mine2::ShardedLruCache;
use num_cpus;

const OP_COUNT:usize = 100000;
const CACHE_SIZE:usize = 256;
const SHARD_COUNT:usize = 16;

fn criterion_benchmark(c: &mut Criterion) {
    let CONCURRENCY:usize = num_cpus::get();
    let capacity_per_shard = (CACHE_SIZE / SHARD_COUNT) as u64;
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(CONCURRENCY)
        .build()
        .unwrap();

    c.bench_function(format!("{OP_COUNT} ops in 1 thread against my_cache(size={CACHE_SIZE})").as_str(), |b| b.iter(|| {
        let mut cache: NaiveLruCache<i32> = NaiveLruCache::new(CACHE_SIZE);
        (0..OP_COUNT).for_each(|_|{
            let item = thread_rng().gen_range(0..200);
            cache.put(item);
        })
    }));

    c.bench_function(format!("{OP_COUNT} ops in 1 thread against concurrent_lru::LruCache(size={capacity_per_shard}x{SHARD_COUNT})").as_str(), |b| b.iter(|| {
        let mut cache: GithubCache1<i32, i32> = GithubCache1::new(capacity_per_shard);//#shard=16 is hardcoded.
        (0..OP_COUNT).for_each(|_|{
            let item = thread_rng().gen_range(0..200);
            cache.get_or_init(item, 1, |x|1);
        })
    }));

    c.bench_function(format!("{OP_COUNT} ops in {CONCURRENCY} threads against concurrent_lru::LruCache(size={capacity_per_shard}x{SHARD_COUNT})").as_str(), |b| b.iter(|| {
        let mut cache: GithubCache1<i32, i32> = GithubCache1::new(capacity_per_shard);//#shard=16 is hardcoded.
        thread_pool.install(||{
            (0..OP_COUNT).into_par_iter().for_each(|_|{
                let item = thread_rng().gen_range(0..200);
                cache.get_or_init(item, 1, |x|1);
            })
        });
    }));

    c.bench_function(format!("{OP_COUNT} ops in 1 thread against lru::LruCache(size={CACHE_SIZE})").as_str(), |b| b.iter(|| {
        let mut cache: GithubCache2<i32, i32> = GithubCache2::new(256);
        (0..OP_COUNT).for_each(|_|{
            let item = thread_rng().gen_range(0..200);
            cache.push(item, item);
        })
    }));

    c.bench_function(format!("{OP_COUNT} ops in {CONCURRENCY} threads against lru::LruCache(size={CACHE_SIZE})").as_str(), |b| b.iter(|| {
        let mut cache: Mutex<GithubCache2<i32, i32>> = Mutex::new(GithubCache2::new(CACHE_SIZE));
        thread_pool.install(||{
            (0..OP_COUNT).into_par_iter().for_each(|_i|{
                let item = thread_rng().gen_range(0..200);
                cache.lock().unwrap().push(item, item);
            });
        });
    }));

    c.bench_function(format!("{OP_COUNT} ops in {CONCURRENCY} threads against my_sharded_cache based on lru::LruCache (size={capacity_per_shard}x{SHARD_COUNT})").as_str(), |b| b.iter(|| {
        let mut cache: ShardedLruCache<i32, i32> = ShardedLruCache::new(SHARD_COUNT, capacity_per_shard as usize);
        thread_pool.install(||{
            (0..OP_COUNT).into_par_iter().for_each(|_i|{
                let item = thread_rng().gen_range(0..200);
                cache.push(item, item);
            });
        });
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
