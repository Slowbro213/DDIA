use bumpalo::Bump;
use lsm::memtable::Memtable;

#[test]
fn put_and_get() {
    let alloc = Bump::new();
    let mut m: Memtable<&str, &str, &Bump> = Memtable::new_in(&alloc);

    let key = "Thanas";
    let value = "Papa";

    m.put(key, value);

    let returned = m.get(&key);

    assert_eq!(returned, Some(Some(value)).as_ref());
}

#[test]
fn put_wrong_and_get() {
    let alloc = Bump::new();
    let mut m: Memtable<&str, &str, &Bump> = Memtable::new_in(&alloc);

    let key = "Thanas";
    let wrong_key = "Thanas2";
    let value = "Papa";

    m.put(key, value);

    let returned = m.get(&wrong_key);

    assert_ne!(returned, Some(Some(value)).as_ref());
    assert_eq!(returned, None);
}

#[test]
fn put_and_rm_and_get() {
    let alloc = Bump::new();
    let mut m: Memtable<&str, &str, &Bump> = Memtable::new_in(&alloc);

    let key = "Thanas";
    let value = "Papa";

    m.put(key, value);
    m.rm(key);

    match m.get(&key).expect("Value could not be found") {
        Some(_) => panic!("Value should have not been found because it was deleted"),
        _ => {}
    };
}
