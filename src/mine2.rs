use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use lru::LruCache;

pub struct ShardedLruCache<K,V> {
    inner: Vec<Mutex<LruCache<K,V>>>,
}

impl<K:Hash+Eq, V> ShardedLruCache<K,V> {
    pub fn new(shard_count: usize, capacity_per_shard: usize) -> Self {
        ShardedLruCache {
            inner: (0..shard_count).map(|x|Mutex::new(LruCache::new(capacity_per_shard))).collect(),
        }
    }
    fn get_shard_id(&self, key: &K) -> usize {
        let mut hash_builder = DefaultHasher::new();
        key.hash(&mut hash_builder);
        let hash_value = hash_builder.finish();
        let shard_id = (hash_value as usize) % self.inner.len();
        shard_id
    }

    pub fn push(&self, key: K, value: V) -> Option<(K,V)> {
        self.inner[self.get_shard_id(&key)].lock().unwrap().deref_mut().push(key, value)
    }
}

unsafe impl<K,V> Send for ShardedLruCache<K,V> {}
unsafe impl<K,V> Sync for ShardedLruCache<K,V> {}

#[cfg(test)]
mod my_test {
    use crate::mine2::ShardedLruCache;

    #[test]
    fn basics() {
        let mut cache : ShardedLruCache<i64,i64> = ShardedLruCache::new(2, 2);
        cache.push(999,999);
        cache.push(888,888);
        cache.push(777,777);
        cache.push(666,666);
    }

}
