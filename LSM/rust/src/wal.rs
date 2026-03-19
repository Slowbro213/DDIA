use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, Read, Write},
    path::Path,
};

pub struct WAL {
    file: File,
}

impl WAL {
    pub fn open(path: &Path) -> Result<Self, io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn append(&mut self, data: &[u8]) -> io::Result<()> {
        let len = data.len() as u32;

        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(&data)
    }

    pub fn iter(&self) -> io::Result<WalReader> {
        WalReader::open(&self.path)
    }

    pub fn sync(&mut self) -> io::Result<()> {
        self.file.sync_data()
    }
}

pub struct WalReader {
    reader: BufReader<File>,
}

impl WalReader {
    pub fn open(path: &Path) -> Result<Self, io::Error> {
        let reader = BufReader::new(File::open(path)?);
        Ok(Self { reader })
    }
}

impl Iterator for WalReader {
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut len_buf = [0u8; 4];

        match self.reader.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return None,
            Err(e) => return Some(Err(e)),
        };

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut data = vec![0; len];

        if let Err(e) = self.reader.read_exact(&mut data) {
            return Some(Err(e));
        }

        Some(Ok(data))
    }
}
