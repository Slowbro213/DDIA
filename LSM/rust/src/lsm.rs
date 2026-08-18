use std::{
    collections::btree_map::Iter,
    fs::{File, OpenOptions},
    io::{self, Write},
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
const WAL_PATH: &str = "wal.txt";
const MEMTABLE_MAX_SIZE: usize = 10000;

pub struct LSM<K: Ord + ToBytes, V: ToBytes + Clone> {
    memtable: Memtable<K, V>,
    sstables: Vec<SSTable<K, V>>,
    wal: File,

    data_dir: String,
}

impl<K: Ord + ToBytes + Clone, V: ToBytes + Clone> LSM<K, V> {
    pub fn new_empty(data_dir: String) -> Result<Self, io::Error> {
        let wal_path = Path::new(&data_dir).join(WAL_PATH);
        let wal = OpenOptions::new()
            .create(true)
            .append(true)
            .open(wal_path)?;
        Ok(Self {
            memtable: Memtable::new(MEMTABLE_MAX_SIZE),
            sstables: Vec::new(),
            data_dir,
            wal,
        })
    }

    pub fn new(data_dir: String) -> Result<Self, io::Error> {
        let heap_path = Path::new(&data_dir).join(Path::new(HEAP_DIR));
        let sparse_index_path = Path::new(&data_dir).join(Path::new(SPARSE_INDEX_DIR));

        let wal_path = Path::new(&data_dir).join(WAL_PATH);
        let mut wal = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(wal_path)?;

        let memtable = Memtable::from_wal(&mut wal, MEMTABLE_MAX_SIZE)?;
        Ok(Self {
            memtable,
            sstables: SSTable::from_data(&heap_path, &sparse_index_path)?,
            data_dir,
            wal,
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
        // Append insert into WAL
        self.append_to_wal(key.clone(), Some(value.clone()))?;

        let val = self.memtable.put(key, value);

        if self.memtable.len() >= self.memtable.max_size {
            self.sstables.push(self.flush()?);
            self.memtable.clear();
        }

        Ok(val)
    }

    pub fn delete(&mut self, key: K) -> Result<Option<V>, io::Error> {
        // Append insert into WAL
        self.append_to_wal(key.clone(), None)?;

        Ok(self.memtable.delete(key))
    }

    pub fn clear(&mut self) {
        self.memtable.clear()
    }

    pub fn flush(&self) -> Result<SSTable<K, V>, io::Error> {
        let heap_path = Path::new(&self.data_dir)
            .join(Path::new(HEAP_DIR))
            .join(Path::new(&self.sstables.len().to_string()));
        let sparse_index_path = Path::new(&self.data_dir)
            .join(Path::new(SPARSE_INDEX_DIR))
            .join(Path::new(&self.sstables.len().to_string()));

        let iter: Peekable<Iter<K, Option<V>>> = self.memtable.map.iter().peekable();
        self.wal.set_len(0)?;
        self.wal.sync_all()?;

        SSTable::from_iter(&heap_path, &sparse_index_path, iter)
    }

    fn append_to_wal(&mut self, key: K, value: Option<V>) -> Result<(), io::Error> {
        let mut buf = Vec::new();

        buf.extend(key.bytes_len().to_le_bytes());
        buf.extend(key.serialize());
        if let Some(v) = value {
            buf.extend(v.bytes_len().to_le_bytes());
            buf.extend(v.serialize());
        } else {
            buf.extend(0_usize.to_le_bytes());
        }

        self.wal.write_all(buf.as_slice())?;
        // Commented out for now
        // self.wal.sync_all()?; 

        Ok(())
    }
}
