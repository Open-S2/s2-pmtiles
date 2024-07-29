#[cfg(feature = "alloc")]
extern crate alloc;

use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// A simple cache system with a maximum size.
/// The key is the offset in the data and the value is the directory entries.
#[derive(Debug, Default)]
pub struct DirCache<K, V> {
    cache: BTreeMap<K, V>,
    order: Vec<K>,
    max_size: usize,
}

impl<K: Ord + Clone, V> DirCache<K, V> {
    /// Creates a new DirCache with the specified maximum size.
    pub fn new(max_size: usize) -> Self {
        DirCache {
            cache: BTreeMap::new(),
            order: Vec::new(),
            max_size,
        }
    }

    /// Inserts a key-value pair into the cache, applying LRU rules.
    pub fn set(&mut self, key: K, dir: V) -> &mut Self {
        // Place the new key at the front of the order list
        self.order.insert(0, key.clone());
        while self.order.len() > self.max_size {
            if let Some(oldest_key) = self.order.pop() {
                self.cache.remove(&oldest_key);
            }
        }
        self.cache.insert(key, dir);
        self
    }

    /// Retrieves a reference to the value corresponding to the key, if it exists,
    /// while also updating its position in the LRU order.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            let key_clone = self.order.remove(pos);
            self.order.insert(0, key_clone);
        }
        self.cache.get(key)
    }

    /// Removes a key from the cache, if it exists.
    pub fn delete(&mut self, key: &K) -> bool {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
        self.cache.remove(key).is_some()
    }
}
