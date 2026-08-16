use std::{
    collections::btree_map::Iter,
    io::{self},
    iter::Peekable,
    path::Path,
};

use crate::{
    memtable::Memtable,
    sstable::{SSTable, ToBytes},
};

impl ToBytes for usize {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn deserialize(v: &Vec<u8>) -> Self {
        usize::from_le_bytes(v.as_slice().try_into().unwrap())
    }
    fn bytes_len() -> u64 {
        size_of::<usize>() as u64
    }
}

const HEAP_DIR: &str = "heap";
const SPARSE_INDEX_DIR: &str = "sparse_index";
const MEMTABLE_MAX_SIZE: usize = 100;

pub struct LSM<K: Ord + ToBytes, V: ToBytes> {
    memtable: Memtable<K, V>,
    sstables: Vec<SSTable<K, V>>,

    data_dir: String,
}

impl<K: Ord + ToBytes + Clone, V: ToBytes + Clone> LSM<K, V> {
    pub fn new(data_dir: String) -> Self {
        Self {
            memtable: Memtable::new(MEMTABLE_MAX_SIZE),
            sstables: Vec::new(),
            data_dir,
        }
    }
    pub fn get(&self, key: &K) -> Result<Option<V>, io::Error> {
        if let Some(value) = self.memtable.get(key) {
            return Ok(Some(value.clone()));
        }

        for sstable in &self.sstables {
            match sstable.get(key)? {
                Some(value) => return Ok(Some(value)),
                None => {}
            };
        }

        Ok(None)
    }

    pub fn put(&mut self, key: K, value: V) -> Result<Option<V>, io::Error> {
        let val = self.memtable.put(key, value);

        if self.memtable.size >= self.memtable.max_size {
            self.sstables.push(self.flush()?);
            self.memtable.clear();
        }

        Ok(val)
    }

    fn flush(&self) -> Result<SSTable<K, V>, io::Error> {
        let heap_path = Path::new(&self.data_dir)
            .join(Path::new(HEAP_DIR))
            .join(Path::new(&self.sstables.len().to_string()));
        let sparse_index_path = Path::new(&self.data_dir)
            .join(Path::new(SPARSE_INDEX_DIR))
            .join(Path::new(&self.sstables.len().to_string()));

        let iter: Peekable<Iter<K, V>> = self.memtable.map.iter().peekable();

        SSTable::from_iter(&heap_path, &sparse_index_path, iter)
    }
}
