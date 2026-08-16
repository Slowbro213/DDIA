use std::collections::btree_map::Iter;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::iter::Peekable;
use std::path::Path;
use std::{fs::File, marker::PhantomData};

use zstd::DEFAULT_COMPRESSION_LEVEL;

const SPARSITY: usize = 10;

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

    pub fn get_offset(&self, key: &K) -> (u64, u64) {
        match self.sparse_keys.binary_search(key) {
            Ok(index) => return (self.sparse_offsets[index], self.sparse_offsets[index + 1]),
            Err(index) => {
                return (self.sparse_offsets[index - 1], self.sparse_offsets[index]);
            }
        }
    }

    pub fn to_buf(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.sparse_keys.len());

        buf.extend(self.sparse_keys.len().to_le_bytes());
        for key in &self.sparse_keys {
            buf.extend(key.serialize());
        }

        for offset in &self.sparse_offsets {
            buf.extend(offset.to_le_bytes());
        }

        buf
    }

    pub fn from_buf(buf: &[u8]) -> Self {
        // Get the length of the keys from the buffer first
        let keys_len = usize::from_le_bytes(buf.split_at(size_of::<usize>()).0.try_into().unwrap());
        let mut sparse_keys = Vec::with_capacity(keys_len);

        // buffer - the beginning usize leaves just the bytes for all of the offsets
        // diving that by the size of the offset leaves the number of offsets
        let offsets_len = (buf.len() - size_of::<usize>()) / size_of::<u64>();
        let mut sparse_offsets = Vec::with_capacity(offsets_len);

        let keys_bytes = &buf[size_of::<usize>()..keys_len * K::bytes_len() as usize];
        let offsets_bytes = &buf[size_of::<usize>() + keys_len * K::bytes_len() as usize..];

        for chunk in keys_bytes.chunks(K::bytes_len() as usize) {
            sparse_keys.push(K::deserialize(&chunk.to_vec()));
        }

        for chunk in offsets_bytes.chunks(size_of::<u64>()) {
            sparse_offsets.push(u64::from_le_bytes(chunk.try_into().unwrap()));
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
    fn bytes_len() -> u64;
}

pub struct SSTableEntry<K: Ord + ToBytes, V: ToBytes> {
    key_value_pairs: Vec<(K, V)>,
}

impl<K: Ord + ToBytes, V: ToBytes> SSTableEntry<K, V> {
    pub fn from_buf(buf: Vec<u8>) -> Self {
        let mut bytes_iter = buf.into_iter();

        let mut key_value_pairs = Vec::new();
        let mut k_buf = Vec::with_capacity(K::bytes_len() as usize);
        let mut v_buf = Vec::with_capacity(V::bytes_len() as usize);
        while let Some(byte) = bytes_iter.next() {
            if k_buf.len() != K::bytes_len() as usize {
                k_buf.push(byte);
                continue;
            } else if v_buf.len() != V::bytes_len() as usize {
                v_buf.push(byte);
                continue;
            } else {
                key_value_pairs.push((K::deserialize(&k_buf), V::deserialize(&v_buf)));
                k_buf.clear();
                v_buf.clear();
                k_buf.push(byte);
            }
        }
        key_value_pairs.push((K::deserialize(&k_buf), V::deserialize(&v_buf)));

        Self {
            key_value_pairs: key_value_pairs,
        }
    }

    pub fn to_buf(pairs: &Vec<(&K, &V)>) -> Vec<u8> {
        let mut buf = Vec::with_capacity(pairs.len());
        for (a, b) in pairs {
            buf.extend(a.serialize());
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
    pub fn new(index: SparseIndex<K>, filepath: String) -> Self {
        return Self {
            index,
            filepath: filepath,
            _marker: PhantomData {},
        };
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

        sparse_index_file.write_all(sparse_index.to_buf().as_slice())?;

        Ok(Self::new(
            sparse_index,
            heap_path.to_string_lossy().into_owned(),
        ))
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, io::Error> {
        let (start, end) = self.index.get_offset(key);

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
}
