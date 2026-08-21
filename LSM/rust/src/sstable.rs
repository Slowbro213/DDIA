use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::collections::btree_map::Iter;
use std::fs::{self, OpenOptions};
use std::hash::Hash;
use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::iter::{Peekable, Zip};
use std::path::{Path, PathBuf};
use std::{fs::File, marker::PhantomData};

use fastbloom::AtomicBloomFilter;
use zstd::DEFAULT_COMPRESSION_LEVEL;

use crate::sparse_index::{self, SparseIndex};
use crate::sstable_entry::SSTableEntry;

const SPARSITY: usize = 100;

pub trait ToBytes {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(v: &[u8]) -> Self;
    fn bytes_len(&self) -> usize;
}

pub struct SSTable<K: Ord + ToBytes + Clone + Hash, V: ToBytes> {
    index: SparseIndex<K>,
    heap: File,
    bloom: fastbloom::AtomicBloomFilter,

    _marker: PhantomData<V>,
}

impl<K: Ord + ToBytes + Clone + Hash, V: ToBytes> SSTable<K, V> {
    fn new(index: SparseIndex<K>, bloom: AtomicBloomFilter, heap: File) -> Self {
        Self {
            index,
            bloom,
            heap,
            _marker: PhantomData {},
        }
    }

    pub fn get(&self, key: &K) -> Result<Option<Option<V>>, io::Error> {
        if !self.bloom.contains(key) {
            return Ok(None);
        }

        let Some((start, end)) = self.index.get_offset(key) else {
            return Ok(None);
        };

        let mut file = self.heap.try_clone()?;

        file.seek(SeekFrom::Start(start))?;
        let mut buf = vec![0u8; (end - start) as usize];
        file.read_exact(&mut buf)?;

        let data = zstd::decode_all(buf.as_slice())?;
        let sstable_entry: SSTableEntry<K, V> = SSTableEntry::from_buf(data);

        Ok(sstable_entry.take(key))
    }

    pub fn from_iter(
        heap_path: &Path,
        sparse_index_path: &Path,
        bloom_path: &Path,
        mut iter: Peekable<Iter<K, Option<V>>>,
    ) -> Result<Self, io::Error> {
        let mut sparse_index = SparseIndex::new();
        let bloom = AtomicBloomFilter::with_false_pos(0.01).expected_items(iter.len());

        let mut heap_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(heap_path)?;
        let mut sparse_index_file = File::create(sparse_index_path)?;
        let mut bloom_file = File::create(bloom_path)?;

        while iter.peek().is_some() {
            let mut pairs: Vec<(&K, &Option<V>)> = Vec::with_capacity(SPARSITY);
            let mut first_key: Option<K> = None;
            for i in 0..SPARSITY {
                match iter.next() {
                    Some(pair) => {
                        if i == 0 {
                            first_key = Some(pair.0.clone());
                        }
                        pairs.push(pair);
                        bloom.insert(pair.0);
                    }
                    None => break,
                }
            }
            let buf = SSTableEntry::to_buf(&pairs);
            let compressed_buf = zstd::encode_all(buf.as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
            heap_file.write_all(compressed_buf.as_slice())?;
            if let Some(key) = first_key
                && let Some(last_offset) = sparse_index.last_offset()
            {
                sparse_index.add_pair(key, *last_offset + compressed_buf.len() as u64);
            }
        }

        let compressed_sparse_index_buf =
            zstd::encode_all(sparse_index.to_buf().as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
        sparse_index_file.write_all(&compressed_sparse_index_buf)?;

        let bloom_buf = bincode::serialize(&bloom).expect("bloom serialization failed");
        let compressed_bloom_buf =
            zstd::encode_all(bloom_buf.as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
        bloom_file.write_all(&compressed_bloom_buf)?;

        Ok(Self::new(sparse_index, bloom, heap_file))
    }

    pub fn from_data(
        heap_path: &Path,
        sparse_index_path: &Path,
        bloom_path: &Path,
    ) -> Result<Vec<Self>, io::Error> {
        if !sparse_index_path.is_dir() || !heap_path.is_dir() {
            return Err(Error::new(ErrorKind::NotFound, "path is not a directory"));
        }

        let sparse_index_dir = fs::read_dir(sparse_index_path)?;
        let heap_dir = fs::read_dir(heap_path)?;
        let bloom_dir = fs::read_dir(bloom_path)?;

        let count = fs::read_dir(sparse_index_path)?.count();

        // Map of paths used in order to pair sparse indexes with their corresponding heap files
        let mut heap_map: Vec<Option<PathBuf>> =
            std::iter::repeat_with(|| None).take(count).collect();
        let mut bloom_map: Vec<Option<PathBuf>> =
            std::iter::repeat_with(|| None).take(count).collect();

        // map out paths by their names
        // names of sparse_index files
        // and heap files are numbers
        for (h_entry, b_entry) in heap_dir.zip(bloom_dir) {
            let (h_entry, b_entry) = (h_entry?, b_entry?);

            if let Some(num_name) = h_entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<usize>().ok())
            {
                heap_map[num_name] = Some(h_entry.path());
            }
            if let Some(num_name) = b_entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<usize>().ok())
            {
                bloom_map[num_name] = Some(b_entry.path());
            }
        }

        let mut sstables: Vec<Option<SSTable<K, V>>> =
            std::iter::repeat_with(|| None).take(count).collect();
        for s_entry in sparse_index_dir {
            let s_entry = s_entry?;
            let s_path = s_entry.path();
            if s_path.is_file() {
                let mut s_file = File::open(s_path)?;

                let mut s_compressed_buf = Vec::new();
                s_file.read_to_end(&mut s_compressed_buf)?;
                let s_buf = zstd::decode_all(s_compressed_buf.as_slice())?;
                let sparse_index: SparseIndex<K> = SparseIndex::from_buf(s_buf)?;

                if let Some((num_name, Some(h_path), Some(b_path))) = s_entry
                    .file_name()
                    .to_str()
                    .and_then(|name| name.parse::<usize>().ok())
                    .and_then(|num_name| heap_map.get_mut(num_name).map(|h| (num_name, h)))
                    .and_then(|(num_name, h)| bloom_map.get_mut(num_name).map(|b| (num_name, h, b)))
                {
                    let mut b_file = File::open(b_path)?;
                    let mut bloom_compressed_buf = Vec::new();
                    b_file.read_to_end(&mut bloom_compressed_buf)?;
                    let bloom_buf = zstd::decode_all(bloom_compressed_buf.as_slice())?;
                    let bloom = bincode::deserialize(&bloom_buf)
                        .expect("problem in deserializing bloom filter");

                    let heap_file = File::open(h_path)?;
                    sstables[num_name] = Some(SSTable::new(sparse_index, bloom, heap_file));
                }
            }
        }

        Ok(sstables.into_iter().flatten().collect())
    }

    pub fn compact(
        heap_path: &Path,
        sparse_index_path: &Path,
        bloom_path: &Path,
        sstables: &mut Vec<Self>,
    ) -> Result<(), io::Error> {
        let mut len = 0;

        for sstable in &mut *sstables {
            len += sstable.index.len();
        }

        let mut heap_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(heap_path)?;
        let mut sparse_index_file = File::create(sparse_index_path)?;
        let mut bloom_file = File::create(bloom_path)?;

        let mut sparse_index = SparseIndex::with_capacity(len);
        let bloom = AtomicBloomFilter::with_false_pos(0.01).expected_items(len);

        let mut offset: u64 = 0;
        // this replaces the original vector with an empty one
        let owned_sstables = std::mem::take(sstables);

        let mut pq = BinaryHeap::with_capacity(owned_sstables.len());
        let mut sstable_iters: Vec<HeapIterEntry<K, V>> = owned_sstables
            .into_iter()
            .map(|sstable| sstable.into_iter().peekable())
            .map(|mut it| HeapIterEntry {
                key: it.peek().expect("iterator shouldnt be empty").0.clone(),
                iter: it,
            })
            .collect();

        for heap_iter_entry in sstable_iters.iter_mut() {
            pq.push(Reverse(heap_iter_entry));
        }

        let mut pairs: Vec<(K, Option<V>)> = Vec::with_capacity(SPARSITY);
        let mut first_key: Option<K> = None;
        loop {
            // get the sstable iter with the next smallest key

            // if this is None then we have reached the end
            // of all sstables
            let Some(Reverse(heap_iter_entry)) = pq.pop() else {
                break;
            };

            // This can only be none if its iterator is empty
            let Some(pair) = heap_iter_entry.update() else {
                continue;
            };

            // put the updated iterator back into the queue
            if heap_iter_entry.iter.peek().is_some() {
                pq.push(Reverse(heap_iter_entry));
            }

            // if the value is none then we have a tombstone
            // so we should skip that entry
            if pair.1.is_none() {
                continue;
            }

            bloom.insert(&pair.0);
            if pairs.len() == 0 {
                first_key = Some(pair.0.clone());
            }
            pairs.push(pair);

            if pairs.len() == SPARSITY {
                let refs_to_pairs = pairs.iter().map(|(x, y)| (x, y)).collect();
                let buf = SSTableEntry::to_buf(&refs_to_pairs);
                let compressed_buf = zstd::encode_all(buf.as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
                heap_file.write_all(compressed_buf.as_slice())?;

                if let Some(key) = first_key {
                    offset += compressed_buf.len() as u64;
                    sparse_index.add_pair(key, offset);
                }

                pairs.clear();
                first_key = None;
            }
        }

        let compressed_sparse_index_buf =
            zstd::encode_all(sparse_index.to_buf().as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
        sparse_index_file.write_all(&compressed_sparse_index_buf)?;

        let bloom_buf = bincode::serialize(&bloom).expect("bloom serialization failed");
        let compressed_bloom_buf =
            zstd::encode_all(bloom_buf.as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
        bloom_file.write_all(&compressed_bloom_buf)?;
        sstables.clear();
        sstables.push(SSTable::new(sparse_index, bloom, heap_file));

        Ok(())
    }
}

impl<K: Ord + ToBytes + Clone + Hash, V: ToBytes> IntoIterator for SSTable<K, V> {
    type Item = (K, Option<V>);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::<K, V> {
            sparse_index_iter: self.index.into_iter(),
            heap: self.heap,
            sstable_entry_iter: None,
        }
    }
}

pub struct IntoIter<K: Ord + ToBytes + Hash, V: ToBytes> {
    sparse_index_iter: sparse_index::IntoIter<K>,
    sstable_entry_iter: Option<Zip<std::vec::IntoIter<K>, std::vec::IntoIter<Option<V>>>>,
    heap: File,
}

impl<K: Ord + ToBytes + Clone + Hash, V: ToBytes> Iterator for IntoIter<K, V> {
    type Item = (K, Option<V>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.sstable_entry_iter.as_mut().and_then(|it| it.next()) {
            return Some(entry);
        } else {
            if let Some((_, (start, end))) = self.sparse_index_iter.next() {
                let mut file = self.heap.try_clone().ok()?;

                file.seek(SeekFrom::Start(start)).ok()?;
                let mut buf = vec![0u8; (end - start) as usize];
                file.read_exact(&mut buf).ok()?;

                let data = zstd::decode_all(buf.as_slice()).ok()?;
                let sstable_entry: SSTableEntry<K, V> = SSTableEntry::from_buf(data);

                let iter = sstable_entry.into_iter();
                self.sstable_entry_iter = Some(iter);
                self.next()
            } else {
                return None;
            }
        }
    }
}

struct HeapIterEntry<K: Ord + Clone + ToBytes + Hash, V: ToBytes> {
    key: K,
    iter: Peekable<IntoIter<K, V>>,
}

impl<K: Ord + Clone + ToBytes + Hash, V: ToBytes> HeapIterEntry<K, V> {
    pub fn update(&mut self) -> Option<(K, Option<V>)> {
        let current = self.iter.next()?;

        if let Some((next_key, _)) = self.iter.peek() {
            self.key = next_key.clone();
        }

        Some(current)
    }
}

impl<K: Ord + Clone + ToBytes + Hash, V: ToBytes> Ord for HeapIterEntry<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: Ord + Clone + ToBytes + Hash, V: ToBytes> PartialOrd for HeapIterEntry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.key.cmp(&other.key))
    }
}

impl<K: Ord + Clone + ToBytes + Hash, V: ToBytes> PartialEq for HeapIterEntry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<K: Ord + Clone + ToBytes + Hash, V: ToBytes> Eq for HeapIterEntry<K, V> {}
