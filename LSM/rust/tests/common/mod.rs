use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A throwaway, uniquely-named data directory for tests that need a real
/// filesystem layout (a `heap/` dir and a `sparse_index/` dir underneath it).
///
/// Each instance gets a random name so parallel tests never collide, and the
/// whole directory tree is removed automatically when it goes out of scope.
#[allow(dead_code)]
pub struct TestDir {
    base: PathBuf,
    pub heap_path: PathBuf,
    pub sparse_index_path: PathBuf,
}

#[allow(dead_code)]
impl TestDir {
    pub fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);

        let base = std::env::temp_dir().join(format!("{prefix}_{nanos}_{count}"));
        let heap_path = base.join("heap");
        let sparse_index_path = base.join("sparse_index");

        fs::create_dir_all(&heap_path).expect("failed to create test heap dir");
        fs::create_dir_all(&sparse_index_path).expect("failed to create test sparse_index dir");

        Self {
            base,
            heap_path,
            sparse_index_path,
        }
    }

    pub fn path(&self) -> &Path {
        &self.base
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
