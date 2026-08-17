use std::collections::btree_map::Iter;
use std::fs;
use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::{fs::File, marker::PhantomData};

use zstd::DEFAULT_COMPRESSION_LEVEL;

use crate::lsm::MEMTABLE_MAX_SIZE;

const SPARSITY: usize = MEMTABLE_MAX_SIZE / 100;

pub struct SparseIndex<K: Ord + ToBytes> {
    sparse_keys: Vec<K>,
    sparse_offsets: Vec<u64>,
}

impl<K: Ord + ToBytes> SparseIndex<K> {
    pub fn new() -> Self {
        let sparse_offsets = vec![0];
        Self {
            sparse_keys: Vec::new(),
            sparse_offsets,
        }
    }

    pub fn add_pair(&mut self, key: K, offset: u64) {
        self.sparse_keys.push(key);
        self.sparse_offsets.push(offset);
    }

    pub fn last_offset(&self) -> Option<&u64> {
        self.sparse_offsets.last()
    }

    pub fn get_offset(&self, key: &K) -> Option<(u64, u64)> {
        match self.sparse_keys.binary_search(key) {
            Ok(index) => Some((self.sparse_offsets[index], self.sparse_offsets[index + 1])),
            Err(index) => {
                if index == 0 {
                    None
                } else {
                    Some((self.sparse_offsets[index - 1], self.sparse_offsets[index]))
                }
            }
        }
    }

    // sparse_indexes layout:
    // keys_len: key (key_len + key)...
    // offsets...
    pub fn to_buf(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.sparse_keys.len());

        buf.extend(self.sparse_keys.len().to_le_bytes());
        for key in &self.sparse_keys {
            buf.extend(key.bytes_len().to_le_bytes());
            buf.extend(key.serialize());
        }

        let mut offsets_itr = self.sparse_offsets.iter();
        offsets_itr.next(); // skip the first 0
        for offset in offsets_itr {
            buf.extend(offset.to_le_bytes());
        }

        buf
    }

    pub fn from_buf(buf: Vec<u8>) -> Self {
        let mut sparse_keys = Vec::new();
        let mut sparse_offsets = vec![0];

        let mut buf_iter = buf.into_iter();
        let keys_len_bytes: Vec<u8> = buf_iter.by_ref().take(size_of::<usize>()).collect();
        let keys_len = usize::from_le_bytes(keys_len_bytes.try_into().unwrap());
        for _ in 0..keys_len {
            let key_len_bytes: Vec<u8> = buf_iter.by_ref().take(size_of::<usize>()).collect();
            let key_len = usize::from_le_bytes(key_len_bytes.try_into().unwrap());
            let key_bytes: Vec<u8> = buf_iter.by_ref().take(key_len).collect();
            let key = K::deserialize(&key_bytes);

            sparse_keys.push(key);
        }

        while let Ok(chunk) = buf_iter.next_chunk::<{ size_of::<u64>() }>() {
            let offset = u64::from_le_bytes(chunk.try_into().unwrap());
            sparse_offsets.push(offset);
        }

        Self {
            sparse_keys,
            sparse_offsets,
        }
    }
}

pub trait ToBytes {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(v: &Vec<u8>) -> Self;
    fn bytes_len(&self) -> usize;
}

pub struct SSTableEntry<K: Ord + ToBytes, V: ToBytes> {
    key_value_pairs: Vec<(K, V)>,
}

impl<K: Ord + ToBytes, V: ToBytes> SSTableEntry<K, V> {
    // layout in buffer should be:
    // key_len , key bytes, value_len, value_bytes
    pub fn from_buf(buf: Vec<u8>) -> Self {
        let mut bytes_iter = buf.into_iter();

        let mut key_value_pairs = Vec::new();
        while let Ok(len_bytes) = bytes_iter.next_chunk::<{ size_of::<usize>() }>() {
            let key_len = usize::from_le_bytes(len_bytes.try_into().unwrap());
            let key_bytes: Vec<u8> = bytes_iter.by_ref().take(key_len).collect();
            let key = K::deserialize(&key_bytes.to_vec());

            let len_bytes: Vec<u8> = bytes_iter.by_ref().take(size_of::<usize>()).collect();
            let value_len = usize::from_le_bytes(len_bytes.try_into().unwrap());
            let value_bytes: Vec<u8> = bytes_iter.by_ref().take(value_len).collect();
            let value = V::deserialize(&value_bytes.to_vec());

            key_value_pairs.push((key, value));
        }

        Self {
            key_value_pairs: key_value_pairs,
        }
    }

    pub fn to_buf(pairs: &Vec<(&K, &V)>) -> Vec<u8> {
        let mut buf = Vec::with_capacity(pairs.len());
        for (a, b) in pairs {
            buf.extend(a.bytes_len().to_le_bytes());
            buf.extend(a.serialize());

            buf.extend(b.bytes_len().to_le_bytes());
            buf.extend(b.serialize());
        }
        buf
    }
}

impl<K: Ord + ToBytes, V: ToBytes> IntoIterator for SSTableEntry<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.key_value_pairs.into_iter()
    }
}

pub struct SSTable<K: Ord + ToBytes, V: ToBytes> {
    index: SparseIndex<K>,
    filepath: String,

    _marker: PhantomData<V>,
}

impl<K: Ord + ToBytes + Clone, V: ToBytes> SSTable<K, V> {
    fn new(index: SparseIndex<K>, filepath: String) -> Self {
        return Self {
            index,
            filepath: filepath,
            _marker: PhantomData {},
        };
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, io::Error> {
        let Some((start, end)) = self.index.get_offset(key) else {
            return Ok(None);
        };

        let mut file = File::open(&self.filepath)?;
        file.seek(SeekFrom::Start(start))?;
        let mut buf = vec![0u8; (end - start) as usize];
        file.read_exact(&mut buf)?;

        let data = zstd::decode_all(buf.as_slice())?;
        let sstable_entry: SSTableEntry<K, V> = SSTableEntry::from_buf(data);

        Ok(sstable_entry
            .into_iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v))
    }

    pub fn from_iter(
        heap_path: &Path,
        sparse_index_path: &Path,
        mut iter: Peekable<Iter<K, V>>,
    ) -> Result<Self, io::Error> {
        let mut sparse_index = SparseIndex::new();

        let mut heap_file = File::create(heap_path)?;
        let mut sparse_index_file = File::create(sparse_index_path)?;

        while iter.peek().is_some() {
            let mut pairs: Vec<(&K, &V)> = Vec::with_capacity(SPARSITY);
            for _ in 0..SPARSITY {
                match iter.next() {
                    Some(pair) => pairs.push(pair),
                    None => break,
                }
            }
            let buf = SSTableEntry::to_buf(&pairs);
            let compressed_buf = zstd::encode_all(buf.as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
            heap_file.write_all(compressed_buf.as_slice())?;
            if let Some(first_pair) = pairs.first() {
                if let Some(last_offset) = sparse_index.last_offset() {
                    sparse_index.add_pair(
                        first_pair.0.clone(),
                        *last_offset + compressed_buf.len() as u64,
                    );
                }
            }
        }

        let compressed_sparse_index_buf =
            zstd::encode_all(sparse_index.to_buf().as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
        sparse_index_file.write_all(&compressed_sparse_index_buf)?;

        Ok(Self::new(
            sparse_index,
            heap_path.to_string_lossy().into_owned(),
        ))
    }

    pub fn from_data(heap_path: &Path, sparse_index_path: &Path) -> Result<Vec<Self>, io::Error> {
        if !sparse_index_path.is_dir() || !heap_path.is_dir() {
            return Err(Error::new(ErrorKind::NotFound, "path is not a directory"));
        }

        let sparse_index_dir = fs::read_dir(sparse_index_path)?;
        let heap_dir = fs::read_dir(heap_path)?;
        let count = fs::read_dir(sparse_index_path)?.count();

        let mut map: Vec<Option<PathBuf>> = vec![None; count];

        for entry in heap_dir {
            let entry = entry?;

            if let Some(num_name) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<usize>().ok())
            {
                map[num_name] = Some(entry.path());
            }
        }

        let mut sstables = Vec::new();
        for s_entry in sparse_index_dir {
            let s_entry = s_entry?;
            let s_path = s_entry.path();
            if s_path.is_file() {
                let mut s_file = File::open(s_path)?;

                let mut s_compressed_buf = Vec::new();
                s_file.read_to_end(&mut s_compressed_buf)?;
                let s_buf = zstd::decode_all(s_compressed_buf.as_slice())?;
                let sparse_index: SparseIndex<K> = SparseIndex::from_buf(s_buf);

                if let Some(h_path) = s_entry
                    .file_name()
                    .to_str()
                    .and_then(|name| name.parse::<usize>().ok())
                    .and_then(|num_name| map[num_name].as_mut())
                {
                    sstables.push(SSTable::new(
                        sparse_index,
                        h_path.to_string_lossy().into_owned(),
                    ));
                }
            }
        }

        Ok(sstables)
    }
}
