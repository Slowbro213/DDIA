use std::{iter::Zip, vec::IntoIter};

use crate::sstable::ToBytes;

pub struct SSTableEntry<K: Ord + ToBytes, V: ToBytes> {
    keys: Vec<K>,
    values: Vec<Option<V>>,
}

impl<K: Ord + ToBytes, V: ToBytes> IntoIterator for SSTableEntry<K, V> {
    type Item = (K, Option<V>);
    type IntoIter = Zip<IntoIter<K>, IntoIter<Option<V>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter().zip(self.values.into_iter())
    }
}

impl<K: Ord + ToBytes, V: ToBytes> SSTableEntry<K, V> {
    pub fn take(mut self, key: &K) -> Option<Option<V>> {
        Some(self.values.swap_remove(self.keys.binary_search(key).ok()?))
    }

    // layout in buffer should be:
    // key_len , key bytes, value_len, value_bytes

    // Breaks in this loop are there to ensure parsing ends when corrupted data
    // is encountered. corrupted data may arise on a write which was interrupted
    // due to a crash
    pub fn from_buf(buf: Vec<u8>) -> Self {
        let mut bytes_iter = buf.into_iter();

        let mut keys = Vec::new();
        let mut values = Vec::new();
        while let Ok(len_bytes) = bytes_iter.next_chunk::<{ size_of::<usize>() }>() {
            if len_bytes.len() != size_of::<usize>() {
                break;
            }
            let key_len = usize::from_le_bytes(len_bytes);
            let key_bytes: Vec<u8> = bytes_iter.by_ref().take(key_len).collect();
            if key_bytes.len() != key_len {
                break;
            }
            let key = K::deserialize(&key_bytes);

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
                let value = V::deserialize(&value_bytes);

                keys.push(key);
                values.push(Some(value));
            } else {
                keys.push(key);
                values.push(None);
            }
        }

        Self { keys, values }
    }

    pub fn to_buf(pairs: &Vec<(&K, &Option<V>)>) -> Vec<u8> {
        let mut buf = Vec::with_capacity(pairs.len());
        for (a, b) in pairs {
            buf.extend(a.bytes_len().to_le_bytes());
            buf.extend(a.serialize());

            if let Some(value) = b {
                buf.extend(value.bytes_len().to_le_bytes());
                buf.extend(value.serialize());
            } else {
                buf.extend((0_usize).to_le_bytes());
            }
        }
        buf
    }
}
