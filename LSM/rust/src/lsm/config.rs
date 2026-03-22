use std::path::Path;

use crate::sstable::config::SSTableConfig;

pub struct LSMConfig<'a> {
    mem_cap: u64,
    data_path: &'a Path,
    sstable_config: SSTableConfig,
}

impl<'a> LSMConfig<'a> {
    pub fn new(mem_cap: u64, data_path: &'a Path, sstable_config: SSTableConfig) -> Self {
        Self {
            mem_cap,
            data_path,
            sstable_config,
        }
    }

    #[inline]
    pub fn data_path(&self) -> &Path {
        self.data_path
    }

    #[inline]
    pub fn mem_cap(&self) -> u64 {
        self.mem_cap
    }

    pub fn sstable_config(&self) -> &SSTableConfig {
        &self.sstable_config
    }
}
