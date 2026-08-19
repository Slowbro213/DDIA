use lsm::sparse_index::SparseIndex;

#[test]
fn get_offset() {
    let mut sparse_index = SparseIndex::new();
    let keys: Vec<usize> = vec![2, 3, 4, 6, 7, 8];
    let offsets: Vec<u64> = vec![1, 2, 3, 4, 5, 6];

    // Put them all
    for (k, v) in keys.into_iter().zip(offsets.into_iter()) {
        sparse_index.add_pair(k, v);
    }

    // simple checks first
    assert_eq!((0, 1), sparse_index.get_offset(&2).unwrap());
    assert_eq!((1, 2), sparse_index.get_offset(&3).unwrap());

    // key is missing checks
    // first
    assert_eq!(None, sparse_index.get_offset(&1));
    // middle
    assert_eq!((2, 3), sparse_index.get_offset(&5).unwrap());
    // last
    assert_eq!((5, 6), sparse_index.get_offset(&8).unwrap());
}
