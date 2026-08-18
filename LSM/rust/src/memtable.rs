use std::{
    collections::{
        BTreeMap,
        btree_map::{IntoIter, Iter},
    },
    fs::File,
    io::{self, Read},
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

    pub fn from_wal(wal: &mut File, max_size: usize) -> Result<Self, io::Error> {
        let mut map = BTreeMap::new();

        let mut wal_buf = Vec::new();
        if 0 == wal.read_to_end(&mut wal_buf)? {
            return Ok(Self { map, max_size });
        }

        // Breaks in this loop are there to ensure parsing ends when corrupted data
        // is encountered. corrupted data may arise on a write which was interrupted
        // due to a crash
        let mut bytes_iter = wal_buf.into_iter();
        while let Ok(len_bytes) = bytes_iter.next_chunk::<{ size_of::<usize>() }>() {
            if len_bytes.len() != size_of::<usize>() {
                break;
            }
            let key_len = usize::from_le_bytes(len_bytes);
            let key_bytes: Vec<u8> = bytes_iter.by_ref().take(key_len).collect();
            if key_bytes.len() != key_len {
                break;
            }
            let key = K::deserialize(&key_bytes.to_vec());

            let len_bytes: Vec<u8> = bytes_iter.by_ref().take(size_of::<usize>()).collect();
            if len_bytes.len() != size_of::<usize>() {
                break;
            }
            let value_len = usize::from_le_bytes(len_bytes.try_into().unwrap());
            if value_len > 0 {
                let value_bytes: Vec<u8> = bytes_iter.by_ref().take(value_len).collect();
                if value_bytes.len() != value_len {
                    break;
                }
                let value = V::deserialize(&value_bytes.to_vec());

                map.insert(key, Some(value));
            } else {
                map.insert(key, None);
            }
        }

        wal.set_len(0)?;
        wal.sync_all()?;

        Ok(Self { map, max_size })
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
