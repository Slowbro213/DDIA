use lsm::lsm::LSM;
mod common;

#[test]
fn basic_usage() {
    let test_dir = common::TestDir::new("data_basic_usage");
    let mut lsm: LSM<usize, usize> = LSM::new_empty(test_dir.path().to_string_lossy().into_owned());

    let mut pairs = Vec::new();

    for i in 0..1000 {
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

    for i in 0..1000 {
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
    let test_dir = common::TestDir::new("data_string_usage");
    let mut lsm: LSM<String, String> =
        LSM::new_empty(test_dir.path().to_string_lossy().into_owned());

    let mut pairs = Vec::new();

    for i in 0..1000 {
        pairs.push((format!("key{}", i), format!("value{}", i)));
    }

    // Put them all
    for (k, v) in pairs.clone() {
        lsm.put(k, v).unwrap();
    }

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
