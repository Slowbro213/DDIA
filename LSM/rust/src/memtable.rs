use core::alloc::Allocator;
use std::collections::BTreeMap;

pub struct Memtable<K: Ord, V, A: Allocator + Clone> {
    map: BTreeMap<K, Option<V>, A>,
}

impl<K: Ord, V, A: Allocator + Clone> Memtable<K, V, A> {
    pub fn new_in(alloc: A) -> Self {
        Self {
            map: BTreeMap::new_in(alloc),
        }
    }
    pub fn get(&self, k: &K) -> Option<&Option<V>> {
        self.map.get(k)
    }
    pub fn put(&mut self, k: K, v: V) -> Option<Option<V>> {
        self.map.insert(k, Some(v))
    }
    pub fn rm(&mut self, k: K) -> Option<Option<V>> {
        self.map.insert(k, None)
    }
    pub fn iter_mut(&mut self) -> std::collections::btree_map::IterMut<'_, K, Option<V>> {
        self.map.iter_mut()
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
}
