use lsm::lsm::LSM;
mod common;

#[test]
fn basic_usage() {
    let test_dir = common::TestDir::new("data_basic_usage");
    let mut lsm: LSM<usize, usize> = LSM::new_empty(test_dir.path().to_string_lossy().into_owned());

    let mut pairs = Vec::new();

    for i in 0..10000 {
        pairs.push((i * 2, i * 2 + 1));
    }

    // Put them all
    for (k, v) in pairs.clone() {
        lsm.put(k, v).unwrap();
    }

    // Get them all
    for (k, v) in pairs.clone() {
        if let Some(value) = lsm.get(&k).unwrap() {
            assert_eq!(v, value);
        } else {
            panic!("lsm get returned None when it should have been Some");
        }
    }
}

#[test]
fn string_usage() {
    let test_dir = common::TestDir::new("data_string_usage");
    let mut lsm: LSM<String, String> =
        LSM::new_empty(test_dir.path().to_string_lossy().into_owned());

    let mut pairs = Vec::new();

    for i in 0..10000 {
        pairs.push((format!("key{}", i), format!("value{}", i)));
    }

    // Put them all
    for (k, v) in pairs.clone() {
        lsm.put(k, v).unwrap();
    }

    // Get them all
    for (k, v) in pairs.clone() {
        if let Some(value) = lsm.get(&k).unwrap() {
            assert_eq!(v, value);
        } else {
            panic!("lsm get returned None when it should have been Some");
        }
    }
}

#[test]
fn writes_persist_and_are_searchable() {
    let test_dir = common::TestDir::new("data_persist_usage");
    let mut lsm: LSM<String, String> =
        LSM::new_empty(test_dir.path().to_string_lossy().into_owned());

    let mut pairs = Vec::new();

    // -1 so that at least one entry is in the memtable
    for i in 0..(10000-1) {
        pairs.push((format!("key{}", i), format!("value{}", i)));
    }

    // Put them all
    for (k, v) in pairs.clone() {
        lsm.put(k, v).unwrap();
    }

    // flush the memtable so all entires are on disk
    lsm.clear().unwrap();

    let new_lsm: LSM<String, String> =
        LSM::new(test_dir.path().to_string_lossy().into_owned()).unwrap();

    // Get them all
    for (k, v) in pairs.clone() {
        if let Some(value) = new_lsm.get(&k).unwrap() {
            assert_eq!(v, value);
        } else {
            panic!("lsm get returned None when it should have been Some");
        }
    }
}

#[test]
fn basic_deleting() {
    let test_dir = common::TestDir::new("basic_deleting");
    let mut lsm: LSM<String, String> =
        LSM::new_empty(test_dir.path().to_string_lossy().into_owned());

    let mut pairs = Vec::new();

    for i in 0..10000 {
        pairs.push((format!("key{}", i), format!("value{}", i)));
    }

    let mut pairs_to_delete = Vec::new();

    for i in 0..1000 {
        pairs_to_delete.push((format!("key{}", i), format!("value{}", i)));
    }

    for i in 2000..3000 {
        pairs_to_delete.push((format!("key{}", i), format!("value{}", i)));
    }

    for i in 7000..8000 {
        pairs_to_delete.push((format!("key{}", i), format!("value{}", i)));
    }

    // Put them all
    for (k, v) in pairs.clone() {
        lsm.put(k, v).unwrap();
    }

    // Delete some pairs
    for (k, _) in pairs_to_delete.clone() {
        lsm.delete(k);
    }

    // Get them all
    for (k, v) in pairs.clone() {
        if pairs_to_delete.contains(&(k.clone(), v.clone())) {
            assert_eq!(lsm.get(&k).unwrap(), None);
        } else if let Some(value) = lsm.get(&k).unwrap() {
            assert_eq!(v, value);
        } else {
            panic!("lsm get returned None when it should have been Some");
        }
    }
}
