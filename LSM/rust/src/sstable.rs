use std::collections::btree_map::Iter;
use std::fs::{self, OpenOptions};
use std::hash::Hash;
use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::{fs::File, marker::PhantomData};

use fastbloom::AtomicBloomFilter;
use zstd::DEFAULT_COMPRESSION_LEVEL;

use crate::sparse_index::SparseIndex;
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
            for _ in 0..SPARSITY {
                match iter.next() {
                    Some(pair) => {
                        pairs.push(pair);
                        bloom.insert(pair.0);
                    }
                    None => break,
                }
            }
            let buf = SSTableEntry::to_buf(&pairs);
            let compressed_buf = zstd::encode_all(buf.as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
            heap_file.write_all(compressed_buf.as_slice())?;
            if let Some(first_pair) = pairs.first()
                && let Some(last_offset) = sparse_index.last_offset()
            {
                sparse_index.add_pair(
                    first_pair.0.clone(),
                    *last_offset + compressed_buf.len() as u64,
                );
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

        let mut offset = 0;
        for sstable in &mut *sstables {
            let mut file = sstable.heap.try_clone()?;
            for (_, (start, end)) in std::mem::take(&mut sstable.index) {
                file.seek(SeekFrom::Start(start))?;
                let mut buf = vec![0u8; (end - start) as usize];
                file.read_exact(&mut buf)?;
                let data = zstd::decode_all(buf.as_slice())?;
                let sstable_entry: SSTableEntry<K, V> = SSTableEntry::from_buf(data);
                let mut iter = sstable_entry.into_iter().peekable();

                while iter.peek().is_some() {
                    let mut pairs: Vec<(K, Option<V>)> = Vec::with_capacity(SPARSITY);
                    let mut actual_entries = 0;
                    while actual_entries < SPARSITY
                        && let Some(pair) = iter.next()
                    {
                        if let Some(_) = pair.1 {
                            bloom.insert(&pair.0);
                            pairs.push((pair.0, pair.1));
                            actual_entries += 1;
                        }
                    }
                    let refs: Vec<(&K, &Option<V>)> =
                        pairs.iter().map(|pair| (&pair.0, &pair.1)).collect();
                    let buf = SSTableEntry::to_buf(&refs);
                    let compressed_buf =
                        zstd::encode_all(buf.as_slice(), DEFAULT_COMPRESSION_LEVEL)?;
                    heap_file.write_all(compressed_buf.as_slice())?;
                    if let Some(first_pair) = pairs.first() {
                        offset += compressed_buf.len();
                        sparse_index.add_pair(first_pair.0.clone(), offset as u64);
                    }
                }
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
