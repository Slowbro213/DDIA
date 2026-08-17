use std::{collections::BTreeMap, panic, path::Path};

use lsm::sstable::SSTable;

mod common;

#[test]
fn search() {
    let data_dir = "./data_sstable_search";
    let heap_path = Path::new(data_dir)
        .join(Path::new("heap"))
        .join(Path::new("0"));
    let sparse_index_path = Path::new(data_dir)
        .join(Path::new("sparse_index"))
        .join(Path::new("0"));

    let mut pairs = Vec::new();
    for i in 0..1000 {
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
                None => {
                    common::delete_files_in_dir(data_dir).unwrap();

                    panic!("SSTable returned None when it should have returned Some");
                }
            },
            Err(err) => {
                common::delete_files_in_dir(data_dir).unwrap();
                panic!("{err}");
            }
        }
    }

    common::delete_files_in_dir(data_dir).unwrap();
}
