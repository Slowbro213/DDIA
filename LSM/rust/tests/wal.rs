use lsm::wal::{WAL, WalReader};
use std::fs::remove_file;
use std::io::ErrorKind;
use std::path::Path;

#[test]
fn append_and_read_data() {
    let path = Path::new("./wal.txt");
    let mut wal = match WAL::open(path) {
        Ok(w) => w,
        Err(_) => panic!("Couldnt open file for WAL"),
    };

    let data = [[0, 1, 1, 0], [1, 0, 0, 1]];

    wal.append(&data[0]).expect("Couldnt append data to WAL");
    wal.append(&data[1]).expect("Couldnt append data to WAL");
    wal.sync().expect("Couldnt sync WAL");

    let wal_reader: WalReader = wal.to_reader().unwrap();
    for (wal_data, dat) in wal_reader.zip(data.iter()) {
        let wal_data = wal_data.expect("Wal Iterator returned None");
        if wal_data.as_slice() != dat {
            panic!("Data {:?} is not the same as in wal {:?}", dat, wal_data);
        }
    }

    wal.clear().expect("Couldnt clear WAL");

    match remove_file(path) {
        Ok(()) => println!("File deleted successfully!"),
        Err(e) => {
            eprintln!("Error deleting file: {}", e);
            match e.kind() {
                ErrorKind::NotFound => eprintln!("-> Error: File not found"),
                ErrorKind::PermissionDenied => eprintln!("-> Error: Permission denied"),
                _ => eprintln!("-> Error: Other I/O error"),
            }
        }
    }
}
