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
    fn deserialize(v: &[u8]) -> Self {
        usize::from_le_bytes(v.try_into().unwrap())
    }
    fn bytes_len(&self) -> usize {
        size_of::<usize>()
    }
}

impl ToBytes for String {
    fn serialize(&self) -> Vec<u8> {
        self.clone().into_bytes()
    }
    fn deserialize(v: &[u8]) -> Self {
        String::from_utf8_lossy_owned(v.to_vec())
    }
    fn bytes_len(&self) -> usize {
        self.len()
    }
}

const HEAP_DIR: &str = "heap";
const SPARSE_INDEX_DIR: &str = "sparse_index";
pub const MEMTABLE_MAX_SIZE: usize = 1000;

pub struct LSM<K: Ord + ToBytes, V: ToBytes + Clone> {
    memtable: Memtable<K, V>,
    sstables: Vec<SSTable<K, V>>,

    data_dir: String,
}

impl<K: Ord + ToBytes + Clone, V: ToBytes + Clone> LSM<K, V> {
    pub fn new_empty(data_dir: String) -> Self {
        Self {
            memtable: Memtable::new(MEMTABLE_MAX_SIZE),
            sstables: Vec::new(),
            data_dir,
        }
    }

    pub fn new(data_dir: String) -> Result<Self, io::Error> {
        let heap_path = Path::new(&data_dir).join(Path::new(HEAP_DIR));
        let sparse_index_path = Path::new(&data_dir).join(Path::new(SPARSE_INDEX_DIR));

        Ok(Self {
            memtable: Memtable::new(MEMTABLE_MAX_SIZE),
            sstables: SSTable::from_data(&heap_path, &sparse_index_path)?,
            data_dir,
        })
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, io::Error> {
        if let Some(result) = self.memtable.get(key) {
            return Ok(result.clone());
        }
        // Not in membtable
        // Iterate from latest to oldest
        for sstable in self.sstables.iter().rev() {
            if let Some(value) = sstable.get(key)? {
                return Ok(Some(value));
            };
        }
        Ok(None)
    }

    pub fn put(&mut self, key: K, value: V) -> Result<Option<V>, io::Error> {
        let val = self.memtable.put(key, value);

        if self.memtable.len() >= self.memtable.max_size {
            self.sstables.push(self.flush()?);
            self.memtable.clear();
        }

        Ok(val)
    }

    pub fn delete(&mut self, key: K) -> Option<V> {
        self.memtable.delete(key)
    }

    pub fn clear(&mut self) -> Result<(), io::Error> {
        self.sstables.push(self.flush()?);
        self.memtable.clear();
        Ok(())
    }

    fn flush(&self) -> Result<SSTable<K, V>, io::Error> {
        let heap_path = Path::new(&self.data_dir)
            .join(Path::new(HEAP_DIR))
            .join(Path::new(&self.sstables.len().to_string()));
        let sparse_index_path = Path::new(&self.data_dir)
            .join(Path::new(SPARSE_INDEX_DIR))
            .join(Path::new(&self.sstables.len().to_string()));

        let iter: Peekable<Iter<K, Option<V>>> = self.memtable.map.iter().peekable();

        SSTable::from_iter(&heap_path, &sparse_index_path, iter)
    }
}
