#![feature(allocator_api, anonymous_lifetime_in_impl_trait)]

use std::{
    alloc::Global,
    fs::{self, File, OpenOptions},
    io::Read,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use lsm::{
    config::Config,
    memtable::Memtable,
    sstable::SSTable,
    traits::{Decode, Encode, SSTableError, ToLeBytes},
};

/// Replace `your_crate_name` with the actual crate name from Cargo.toml.
///
/// These tests assume the crate-local traits have the following semantics,
/// based on sstable.rs usage:
/// - Encode::encode_len() -> usize
/// - Encode::encode_into(&mut Vec<u8>)
/// - Decode::decode_from(&mut &[u8]) -> Result<Self, SSTableError>
/// - ToLeBytes::to_le_bytes() -> something usable as a byte slice

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TestKey(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestValue(String);

impl Encode for TestKey {
    fn encode_len(&self) -> usize {
        std::mem::size_of::<u64>()
    }

    fn encode_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.0.to_le_bytes());
    }
}

impl Decode for TestKey {
    fn decode_from(input: &mut &[u8]) -> Result<Self, SSTableError> {
        let mut bytes = [0u8; 8];
        input.read_exact(&mut bytes)?;
        Ok(Self(u64::from_le_bytes(bytes)))
    }
}

impl ToLeBytes for TestKey {
    type Bytes = [u8; 8];

    fn to_le_bytes(&self) -> Self::Bytes {
        self.0.to_le_bytes()
    }
}

impl Encode for TestValue {
    fn encode_len(&self) -> usize {
        self.0.as_bytes().len()
    }

    fn encode_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.0.as_bytes());
    }
}

impl Decode for TestValue {
    fn decode_from(input: &mut &[u8]) -> Result<Self, SSTableError> {
        let mut out = Vec::new();
        input.read_to_end(&mut out)?;
        let s = String::from_utf8(out).map_err(|e| {
            SSTableError::IO(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;
        Ok(Self(s))
    }
}

fn unique_test_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let dir = std::env::temp_dir().join(format!("sstable-tests-{name}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn build_table(name: &str, pairs: impl IntoIterator<Item = (u64, &str)>) -> SSTable<TestKey> {
    let dir = unique_test_dir(name);
    let segment_path = dir.join("segment.db");
    let index_path = dir.join("index.db");

    let segment_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(segment_path)
        .expect("Couldnt create segment file");

    let index_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(index_path)
        .expect("Couldnt create index file");

    let mut memtable = Memtable::<TestKey, TestValue, Global>::new_in(Global);
    for (k, v) in pairs {
        memtable.put(TestKey(k), TestValue(v.to_owned()));
    }

    let cfg = Config::new(&dir, true, 9);

    SSTable::new(segment_file, index_file, &cfg, &mut memtable).unwrap()
}

#[test]
fn builds_sstable_from_memtable() {
    let table = build_table(
        "builds_sstable_from_memtable",
        [(1, "one"), (2, "two"), (3, "three")],
    );

    // Construction succeeding is the primary behavior under test here.
    // The returned table should be usable and own its backing files.
    assert!(
        !table.sparse_index_keys().is_empty(),
        "sparse index should not be empty"
    );
    assert_eq!(
        table.sparse_index_keys().len(),
        table.sparse_index_offsets().len(),
        "sparse index keys/offsets length mismatch"
    );
}

#[test]
fn gets_existing_values() {
    let mut table = build_table(
        "gets_existing_values",
        [(10, "ten"), (20, "twenty"), (30, "thirty")],
    );

    assert_eq!(
        table.get::<TestValue>(&TestKey(10)).unwrap(),
        Some(TestValue("ten".into()))
    );
    assert_eq!(
        table.get::<TestValue>(&TestKey(20)).unwrap(),
        Some(TestValue("twenty".into()))
    );
    assert_eq!(
        table.get::<TestValue>(&TestKey(30)).unwrap(),
        Some(TestValue("thirty".into()))
    );
}

#[test]
fn returns_none_for_missing_key() {
    let mut table = build_table(
        "returns_none_for_missing_key",
        [(1, "one"), (3, "three"), (5, "five")],
    );

    assert_eq!(table.get::<TestValue>(&TestKey(2)).unwrap(), None);
    assert_eq!(table.get::<TestValue>(&TestKey(4)).unwrap(), None);
    assert_eq!(table.get::<TestValue>(&TestKey(99)).unwrap(), None);
}

#[test]
fn can_read_first_and_last_keys() {
    let mut table = build_table(
        "can_read_first_and_last_keys",
        [(1, "alpha"), (2, "beta"), (3, "gamma"), (4, "delta")],
    );

    assert_eq!(
        table.get::<TestValue>(&TestKey(1)).unwrap(),
        Some(TestValue("alpha".into()))
    );
    assert_eq!(
        table.get::<TestValue>(&TestKey(4)).unwrap(),
        Some(TestValue("delta".into()))
    );
}

#[test]
fn supports_many_records_across_multiple_blocks() {
    let dir = unique_test_dir("supports_many_records_across_multiple_blocks");
    let segment_path = dir.join("segment.db");
    let index_path = dir.join("index.db");

    let segment_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(segment_path)
        .expect("Couldnt create segment file");

    let index_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(index_path)
        .expect("Couldnt create index file");

    let mut memtable = Memtable::<TestKey, TestValue, Global>::new_in(Global);

    // Large values to force multiple compressed blocks.
    for i in 0..400u64 {
        let payload = format!("value-{i}-{}", "x".repeat(512));
        memtable.put(TestKey(i), TestValue(payload));
    }

    let cfg = Config::new(&dir, true, 1);
    let mut table = SSTable::new(segment_file, index_file, &cfg, &mut memtable).unwrap();

    assert_eq!(
        table.get::<TestValue>(&TestKey(0)).unwrap(),
        Some(TestValue(format!("value-0-{}", "x".repeat(512))))
    );
    assert_eq!(
        table.get::<TestValue>(&TestKey(199)).unwrap(),
        Some(TestValue(format!("value-199-{}", "x".repeat(512))))
    );
    assert_eq!(
        table.get::<TestValue>(&TestKey(399)).unwrap(),
        Some(TestValue(format!("value-399-{}", "x".repeat(512))))
    );
    assert_eq!(table.get::<TestValue>(&TestKey(500)).unwrap(), None);
}

#[test]
fn index_file_is_created_and_nonempty() {
    let dir = unique_test_dir("index_file_is_created_and_nonempty");
    let segment_path = dir.join("segment.db");
    let index_path = dir.join("index.db");

    let segment_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(segment_path)
        .expect("Couldnt create segment file");

    let index_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(index_path.clone())
        .expect("Couldnt create index file");

    let mut memtable = Memtable::<TestKey, TestValue, Global>::new_in(Global);
    memtable.put(TestKey(1), TestValue("one".into()));
    memtable.put(TestKey(2), TestValue("two".into()));
    memtable.put(TestKey(3), TestValue("three".into()));

    let cfg = Config::new(&dir, true, 1);
    let _table = SSTable::new(segment_file, index_file, &cfg, &mut memtable).unwrap();

    let metadata = fs::metadata(index_path).unwrap();
    assert!(metadata.len() > 0, "index file should not be empty");
}

#[test]
fn segment_file_is_created_and_nonempty() {
    let dir = unique_test_dir("segment_file_is_created_and_nonempty");
    let segment_path = dir.join("segment.db");
    let index_path = dir.join("index.db");

    let segment_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(segment_path.clone())
        .expect("Couldnt create segment file");

    let index_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(index_path)
        .expect("Couldnt create index file");

    let mut memtable = Memtable::<TestKey, TestValue, Global>::new_in(Global);
    memtable.put(TestKey(1), TestValue("one".into()));

    let cfg = Config::new(&dir, true, 1);
    let _table = SSTable::new(segment_file, index_file, &cfg, &mut memtable).unwrap();

    let metadata = fs::metadata(segment_path).unwrap();
    assert!(metadata.len() > 0, "segment file should not be empty");
}

#[test]
fn get_before_smallest_key_searches_first_block_and_returns_none() {
    let mut table = build_table(
        "get_before_smallest_key_searches_first_block_and_returns_none",
        [(10, "ten"), (20, "twenty"), (30, "thirty")],
    );

    assert_eq!(table.get::<TestValue>(&TestKey(1)).unwrap(), None);
}
