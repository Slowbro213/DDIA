use crate::generated::code::{AddUserRequest, BinarySerializable};

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
}
