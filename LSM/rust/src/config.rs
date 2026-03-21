use std::path::Path;

pub struct Config<'a> {
    compression_enabled: bool,
    compression_level: i32,
    data_path: &'a Path,
}

impl<'a> Config<'a> {
    pub fn new(
        data_path: &'a std::path::Path,
        compression_enabled: bool,
        compression_level: i32,
    ) -> Self {
        Self {
            compression_enabled,
            compression_level,
            data_path,
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

    #[inline]
    pub fn data_path(&self) -> &Path {
        self.data_path
    }
}
