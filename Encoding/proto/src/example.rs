use crate::generated::code::{AddUserRequest, BinarySerializable};
use std::fs;

pub fn run_example() {
    let req = AddUserRequest {
        name: "Alice".to_string(),
        surname: "Smith".to_string(),
        age: 25,
    };

    let mut buf = Vec::new();
    req.encode(&mut buf).unwrap();

    let decoded = AddUserRequest::decode(&mut &buf[..]).unwrap();

    println!("encoded bytes: {:?}", buf);
    println!("decoded: {:?}", decoded);

    let long_text = fs::read_to_string("./src/assets/long_text.txt").expect("Couldnt read file");

    println!("{long_text}");

    let req = AddUserRequest {
        name: long_text,
        surname: "Smith".to_string(),
        age: 25,
    };

    let mut buf = Vec::new();
    req.encode(&mut buf).unwrap();

    let decoded = AddUserRequest::decode(&mut &buf[..]).unwrap();

    println!("encoded bytes: {:?}", buf);
    println!("decoded: {:?}", decoded);
}
