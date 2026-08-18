use std::collections::{
    BTreeMap,
    btree_map::{IntoIter, Iter},
};

use crate::sstable::ToBytes;

pub struct Memtable<K: Ord + ToBytes, V: ToBytes + Clone> {
    pub map: BTreeMap<K, Option<V>>,
    pub max_size: usize,
}

impl<K: Ord + ToBytes, V: ToBytes + Clone> Memtable<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            map: BTreeMap::new(),
            max_size,
        }
    }

    pub fn get(&self, key: &K) -> Option<&Option<V>> {
        self.map.get(key)
    }

    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, Some(value))?
    }

    pub fn delete(&mut self, key: K) -> Option<V> {
        self.map.insert(key, None)?
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl<K: Ord + ToBytes, V: ToBytes + Clone> IntoIterator for Memtable<K, V> {
    type Item = (K, Option<V>);
    type IntoIter = IntoIter<K, Option<V>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a, K: Ord + ToBytes, V: ToBytes + Clone> IntoIterator for &'a Memtable<K, V> {
    type Item = (&'a K, &'a Option<V>);
    type IntoIter = Iter<'a, K, Option<V>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}
