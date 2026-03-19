use std::path::Path;

pub struct Config<'a> {
    compression_enabled: bool,
    compression_level: u32,
    data_path: &'a Path,
}
