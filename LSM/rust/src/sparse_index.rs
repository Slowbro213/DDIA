use std::io::{self, Error, ErrorKind};

use crate::sstable::ToBytes;

pub struct SparseIndex<K: Ord + ToBytes> {
    sparse_keys: Vec<K>,
    sparse_offsets: Vec<u64>,
}

impl<K: Ord + ToBytes> Default for SparseIndex<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + ToBytes> SparseIndex<K> {
    pub fn new() -> Self {
        let sparse_offsets = vec![0];
        Self {
            sparse_keys: Vec::new(),
            sparse_offsets,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut sparse_offsets = Vec::with_capacity(capacity + 1);
        sparse_offsets.push(0);
        Self {
            sparse_keys: Vec::with_capacity(capacity),
            sparse_offsets,
        }
    }

    pub fn add_pair(&mut self, key: K, offset: u64) {
        self.sparse_keys.push(key);
        self.sparse_offsets.push(offset);
    }

    pub fn len(&self) -> usize {
        self.sparse_keys.len()
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

    pub fn from_buf(buf: Vec<u8>) -> Result<Self, io::Error> {
        let mut sparse_keys = Vec::new();
        let mut sparse_offsets = vec![0];

        // Errors in this loop are there to ensure parsing ends when corrupted data
        // is encountered. corrupted data may arise on a write which was interrupted
        // due to a crash
        let mut buf_iter = buf.into_iter();
        let keys_len_bytes: Vec<u8> = buf_iter.by_ref().take(size_of::<usize>()).collect();
        if keys_len_bytes.len() != size_of::<usize>() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "SSTableEntry disk data for keys may be corrupted",
            ));
        }
        let keys_len = usize::from_le_bytes(keys_len_bytes.try_into().unwrap());
        for _ in 0..keys_len {
            let key_len_bytes: Vec<u8> = buf_iter.by_ref().take(size_of::<usize>()).collect();
            if key_len_bytes.len() != size_of::<usize>() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "SSTableEntry disk data for keys may be corrupted",
                ));
            }
            let key_len = usize::from_le_bytes(key_len_bytes.try_into().unwrap());
            let key_bytes: Vec<u8> = buf_iter.by_ref().take(key_len).collect();
            if key_bytes.len() != key_len {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "SSTableEntry disk data for keys may be corrupted",
                ));
            }
            let key = K::deserialize(&key_bytes);

            sparse_keys.push(key);
        }

        while let Ok(chunk) = buf_iter.next_chunk::<{ size_of::<u64>() }>() {
            if chunk.len() != size_of::<u64>() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "SSTableEntry disk data for offsets may be corrupted",
                ));
            }
            let offset = u64::from_le_bytes(chunk);
            sparse_offsets.push(offset);
        }

        Ok(Self {
            sparse_keys,
            sparse_offsets,
        })
    }
}

impl<K: Ord + ToBytes> IntoIterator for SparseIndex<K> {
    type Item = (K, (u64, u64));
    type IntoIter = IntoIter<K>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            index: 0,
            sparse_keys: self.sparse_keys.into_iter(),
            sparse_offsets: self.sparse_offsets,
        }
    }
}

pub struct IntoIter<K: Ord + ToBytes> {
    index: usize,
    sparse_keys: std::vec::IntoIter<K>,
    sparse_offsets: Vec<u64>,
}

impl<K: Ord + ToBytes> Iterator for IntoIter<K> {
    type Item = (K, (u64, u64));

    fn next(&mut self) -> Option<Self::Item> {
        let result = Some((
            self.sparse_keys.next()?,
            (
                self.sparse_offsets[self.index],
                self.sparse_offsets[self.index + 1],
            ),
        ));
        self.index += 1;
        result
    }
}
