use std::collections::BTreeMap;

use lsm::sstable::SSTable;

mod common;

#[test]
fn search() {
    let test_dir = common::TestDir::new("data_sstable_search");
    let heap_path = test_dir.heap_path.join("0");
    let sparse_index_path = test_dir.sparse_index_path.join("0");

    let mut pairs = Vec::new();
    for i in 0..10000 {
        pairs.push((i * 2, i * 2 + 1));
    }

    let mut map = BTreeMap::new();
    for (k, v) in pairs.clone() {
        map.insert(k, v);
    }

    let sstable =
        SSTable::from_iter(&heap_path, &sparse_index_path, map.iter().peekable()).unwrap();
    for (k, v) in pairs.clone() {
        match sstable.get(&k) {
            Ok(result) => match result {
                Some(value) => assert_eq!(v, value),
                None => panic!("SSTable returned None when it should have returned Some"),
            },
            Err(err) => panic!("{err}"),
        }
    }
}
