use std::path::Path;

use lsm::wal::WAL;

#[test]
fn append_data() {
    let mut wal: WAL = match WAL::open(Path::new("./wal.txt")) {
        Ok(w) => w,
        _ => panic!("Couldnt open file for WAL"),
    };

    let data1 = [0, 1, 1, 0];
    let data2 = [1, 0, 0, 1];

    match wal.append(&data1) {
        Ok(()) => {}
        _ => panic!("Couldnt open file for WAL"),
    };
    match wal.append(&data2) {
        Ok(()) => {}
        _ => panic!("Couldnt open file for WAL"),
    };
}

#[test]
fn append_and_read_data() {
    let mut wal: WAL = match WAL::open(Path::new("./wal.txt")) {
        Ok(w) => w,
        _ => panic!("Couldnt open file for WAL"),
    };

    let data = [[0, 1, 1, 0], [1, 0, 0, 1]];

    wal.append(&data[0]).unwrap();
    wal.append(&data[1]).unwrap();

    for 
}
