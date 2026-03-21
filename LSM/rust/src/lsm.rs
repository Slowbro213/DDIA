use core::{alloc::Allocator, hash::Hash};

use crate::{
    config::Config,
    memtable::Memtable,
    sstable::SSTable,
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
    pub fn new<F: AllocatorFactory<A>>(factory: F, c: &Config) -> Result<Self, SSTableError> {
        let alloc1 = factory.new_allocator();
        let alloc2 = factory.new_allocator();

        let primary_memtable = Memtable::new_in(alloc1);
        let secondary_memtable = Memtable::new_in(alloc2);
        let sstables = Vec::new();
        let wal_path = c.data_path().join("wal.log");
        let wal = WAL::open(&wal_path)?;

        Ok(Self {
            primary_memtable,
            secondary_memtable,
            sstables,
            wal,
        })
    }
}
