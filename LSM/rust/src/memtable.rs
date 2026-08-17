use std::collections::{
    BTreeMap,
    btree_map::{IntoIter, Iter},
};

use crate::sstable::ToBytes;

pub struct Memtable<K: Ord + ToBytes, V: ToBytes> {
    pub map: BTreeMap<K, V>,
    pub size: usize,
    pub max_size: usize,
}

impl<K: Ord + ToBytes, V: ToBytes> Memtable<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            map: BTreeMap::new(),
            size: 0,
            max_size,
        }
    }

    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.size += 1;
        self.map.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.size = 0;
    }
}

impl<K: Ord + ToBytes, V: ToBytes> IntoIterator for Memtable<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a, K: Ord + ToBytes, V: ToBytes> IntoIterator for &'a Memtable<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}
