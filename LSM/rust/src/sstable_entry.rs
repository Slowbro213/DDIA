use crate::sstable::ToBytes;

pub struct SSTableEntry<K: Ord + ToBytes, V: ToBytes> {
    key_value_pairs: Vec<(K, Option<V>)>,
}

impl<K: Ord + ToBytes, V: ToBytes> SSTableEntry<K, V> {
    // layout in buffer should be:
    // key_len , key bytes, value_len, value_bytes

    // Breaks in this loop are there to ensure parsing ends when corrupted data
    // is encountered. corrupted data may arise on a write which was interrupted
    // due to a crash
    pub fn from_buf(buf: Vec<u8>) -> Self {
        let mut bytes_iter = buf.into_iter();

        let mut key_value_pairs = Vec::new();
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

                key_value_pairs.push((key, Some(value)));
            } else {
                key_value_pairs.push((key, None));
            }
        }

        Self { key_value_pairs }
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

impl<K: Ord + ToBytes, V: ToBytes> IntoIterator for SSTableEntry<K, V> {
    type Item = (K, Option<V>);
    type IntoIter = std::vec::IntoIter<(K, Option<V>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.key_value_pairs.into_iter()
    }
}
