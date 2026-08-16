use std::{
    collections::BTreeMap,
    fs::{self},
    panic,
    path::Path,
};

use lsm::sstable::SSTable;

fn delete_files_in_dir(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

#[test]
fn search() {
    let data_dir = "./data";
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
                    delete_files_in_dir(&heap_path).unwrap();
                    delete_files_in_dir(&sparse_index_path).unwrap();
                    panic!("SSTable returned None when it should have returned Some");
                }
            },
            Err(err) => {
                delete_files_in_dir(&heap_path).unwrap();
                delete_files_in_dir(&sparse_index_path).unwrap();
                panic!("{err}");
            }
        }
    }

    delete_files_in_dir(&heap_path).unwrap();
    delete_files_in_dir(&sparse_index_path).unwrap();
}
