#[cfg(feature = "alloc")]
extern crate alloc;

use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// A simple cache system with a maximum size.
/// The key is the offset in the data and the value is the directory entries.
#[derive(Debug, Default, PartialEq)]
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

    /// Returns the number of elements in the cache.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Inserts a key-value pair into the cache, applying LRU rules.
    pub fn set(&mut self, key: K, dir: V) {
        if let Some(pos) = self.order.iter().position(|k| *k == key) {
            self.order.remove(pos);
        }
        // Place the new key at the front of the order list
        self.order.insert(0, key.clone());
        while self.order.len() > self.max_size {
            let last = self.order.pop().unwrap();
            self.delete(&last);
        }
        self.cache.insert(key, dir);
    }

    /// Retrieves a reference to the value corresponding to the key, if it exists,
    /// while also updating its position in the LRU order.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(pos) = self.order.iter().position(|k| *k == *key) {
            self.order.remove(pos);
            self.order.insert(0, key.clone());
        }
        self.cache.get(key)
    }

    /// Removes a key from the cache, if it exists.
    pub fn delete(&mut self, key: &K) -> bool {
        self.cache.remove(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality() {
        let mut cache = DirCache::<u32, u32>::new(3);

        cache.set(1, 2);
    
        assert_eq!(cache.get(&1), Some(&2));
        assert!(cache.delete(&1));
        assert!(!cache.delete(&1));
    }

    #[test]
    fn test_max_size() {
        let mut cache = DirCache::<u32, u32>::new(5);
        assert!(cache.is_empty());
        cache.set(1, 2);
        cache.set(2, 3);
        cache.set(3, 4);
        cache.set(4, 5);
        cache.set(5, 6);
        cache.set(6, 7);
        cache.set(7, 8);

        assert_eq!(cache, DirCache::<u32, u32>{
            cache: BTreeMap::from([(3, 4), (4, 5), (5, 6), (6, 7), (7, 8)]),
            order: vec![7, 6, 5, 4, 3],
            max_size: 5
        });

        cache.set(5, 9);

        assert_eq!(cache, DirCache::<u32, u32>{
            cache: BTreeMap::from([(3, 4), (4, 5), (5, 9), (6, 7), (7, 8)]),
            order: vec![5, 7, 6, 4, 3],
            max_size: 5
        });
        
        assert_eq!(cache.len(), 5);
        assert!(!cache.is_empty());
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&4));

        assert_eq!(cache, DirCache::<u32, u32>{
            cache: BTreeMap::from([(3, 4), (4, 5), (5, 9), (6, 7), (7, 8)]),
            order: vec![3, 5, 7, 6, 4],
            max_size: 5
        });
    }
}
