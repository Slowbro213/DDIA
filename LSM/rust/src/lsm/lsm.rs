use core::{alloc::Allocator, hash::Hash};
use std::path::Path;

use crate::{
    lsm::config::LSMConfig,
    memtable::Memtable,
    sstable::{SSTable, config::SSTableConfig},
    traits::{Decode, Encode, SSTableError, ToLeBytes},
    wal::WAL,
};

pub trait AllocatorFactory<A: Allocator + Clone> {
    fn new_allocator(&self) -> A;
}

pub struct LSM<
    K: Hash + Ord + Clone + Encode + Decode + ToLeBytes,
    V: Encode + Decode,
    A: Allocator + Clone,
> {
    primary_memtable: Memtable<K, V, A>,
    secondary_memtable: Memtable<K, V, A>,

    sstables: Vec<SSTable<K>>,
    wal: WAL,
}

impl<K: Hash + Ord + Clone + Encode + Decode + ToLeBytes, V: Encode + Decode, A: Allocator + Clone>
    LSM<K, V, A>
{
    pub fn new<F: AllocatorFactory<A>>(factory: F, c: &LSMConfig) -> Result<Self, SSTableError> {
        let alloc1 = factory.new_allocator();
        let alloc2 = factory.new_allocator();

        let mut primary_memtable = Memtable::new_in(alloc1);
        let secondary_memtable = Memtable::new_in(alloc2);
        let sstables = Vec::new();
        let wal_path = c.data_path().join("wal.log");
        let wal = WAL::open(&wal_path)?;

        let wal_reader = wal.to_reader()?;

        for data in wal_reader {
            let data = data?;
            let key = K::decode_from(&mut data.as_slice())?;
            let key_len = key.encode_len();
            let mut slice = &data[key_len..];
            let value = V::decode_from(&mut slice)?;
            primary_memtable.put(key, value);
        }

        Ok(Self {
            primary_memtable,
            secondary_memtable,
            sstables,
            wal,
        })
    }
}
