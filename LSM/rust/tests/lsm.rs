//use lsm::lsm::LSM;

// #[test]
// fn basic_usage() {
//     let data_dir = "./data";
//     let mut lsm: LSM<usize, usize> = LSM::new(String::from(data_dir));
//
//     let mut pairs = Vec::new();
//
//     for i in 0..1000 {
//         pairs.push((i * 2, i * 2 + 1));
//     }
//
//     // Put them all
//     for (k, v) in pairs.clone() {
//         lsm.put(k, v).unwrap();
//     }
//
//     // Get them all
//     for (k, v) in pairs.clone() {
//         if let Some(value) = lsm.get(&k).unwrap() {
//             assert_eq!(v, value);
//         } else {
//             panic!("lsm get returned None when it should have been Some");
//         }
//     }
// }
