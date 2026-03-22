pub struct SSTableConfig {
    pub compression_enabled: bool,
    pub compression_level: i32,
}

impl SSTableConfig {
    pub fn new(compression_enabled: bool, compression_level: i32) -> Self {
        Self {
            compression_enabled,
            compression_level,
        }
    }
    #[inline]
    pub fn compression_enabled(&self) -> bool {
        self.compression_enabled
    }

    #[inline]
    pub fn compression_level(&self) -> i32 {
        self.compression_level
    }
}
