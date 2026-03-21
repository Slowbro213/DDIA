use core::{alloc::Allocator, result::Result};
use std::{
    fs::File,
    io::{BufWriter, Cursor, IntoInnerError, Read, Seek, SeekFrom, Write},
};
use zstd::bulk;

use crate::{
    config::Config,
    memtable::Memtable,
    traits::{Decode, Encode, SSTableError, ToLeBytes},
};

fn read_usize(cursor: &mut Cursor<&[u8]>) -> Result<usize, SSTableError> {
    let mut buf = [0u8; std::mem::size_of::<usize>()];
    cursor.read_exact(&mut buf)?;
    Ok(usize::from_le_bytes(buf))
}

impl From<std::io::Error> for SSTableError {
    fn from(err: std::io::Error) -> Self {
        SSTableError::IO(err)
    }
}

impl From<IntoInnerError<BufWriter<File>>> for SSTableError {
    fn from(err: IntoInnerError<BufWriter<File>>) -> Self {
        SSTableError::IntoInner(err)
    }
}

const BUF_LEN: usize = 1 << 16;

pub struct SSTable<K: Ord> {
    segment_file: File,
    _index_file: File,
    sparse_index_keys: Vec<K>,
    sparse_index_offsets: Vec<u64>,

    buffer: Vec<u8>,
}

impl<K: Ord + Clone + Encode + Decode + ToLeBytes> SSTable<K> {
    fn flush_block(
        segment_writer: &mut BufWriter<File>,
        index_writer: &mut BufWriter<File>,
        sparse_index_keys: &mut Vec<K>,
        sparse_index_offsets: &mut Vec<u64>,
        index_key: K,
        buffer: &mut Vec<u8>,
        compression_level: i32,
    ) -> Result<(), SSTableError> {
        let block_offset = segment_writer.stream_position()? as u64;

        index_writer.write_all(index_key.to_le_bytes().as_ref())?;
        index_writer.write_all(&block_offset.to_le_bytes())?;

        sparse_index_keys.push(index_key);
        sparse_index_offsets.push(block_offset);

        let compressed = bulk::compress(buffer.as_slice(), compression_level)?;

        segment_writer.write_all(&compressed)?;

        buffer.clear();
        Ok(())
    }
    pub fn new<V: Encode, A: Allocator + Clone>(
        segment_file: File,
        index_file: File,
        c: &Config,
        m: &mut Memtable<K, V, A>,
    ) -> Result<Self, SSTableError> {
        let mut sparse_index_keys = Vec::new();
        let mut sparse_index_offsets = Vec::new();

        let mut segment_writer = BufWriter::new(segment_file);
        let mut index_writer = BufWriter::new(index_file);

        let mut index_key: Option<K> = None;
        let mut buffer = Vec::with_capacity(BUF_LEN);

        for (key, value) in m.iter_mut() {
            let key = key.clone();

            if index_key.is_none() {
                index_key = Some(key.clone());
            }

            let key_len = key.encode_len();
            let val_len = value.as_ref().map(|v| v.encode_len()).unwrap_or(0);

            let record_len =
                std::mem::size_of::<usize>() + key_len + std::mem::size_of::<usize>() + val_len;

            if !buffer.is_empty() && buffer.len() + record_len > BUF_LEN {
                let first_key = index_key.take().unwrap();
                Self::flush_block(
                    &mut segment_writer,
                    &mut index_writer,
                    &mut sparse_index_keys,
                    &mut sparse_index_offsets,
                    first_key,
                    &mut buffer,
                    c.compression_level(),
                )?;
                index_key = Some(key.clone());
            }

            key_len.encode_into(&mut buffer);
            key.encode_into(&mut buffer);

            match value {
                Some(val) => {
                    val.encode_len().encode_into(&mut buffer);
                    val.encode_into(&mut buffer);
                }
                _ => {
                    0usize.encode_into(&mut buffer);
                }
            }
        }

        if let Some(first_key) = index_key.take() {
            Self::flush_block(
                &mut segment_writer,
                &mut index_writer,
                &mut sparse_index_keys,
                &mut sparse_index_offsets,
                first_key,
                &mut buffer,
                c.compression_level(),
            )?;
        }

        let segment_file = segment_writer.into_inner()?;
        let index_file = index_writer.into_inner()?;
        segment_file.sync_all()?;
        index_file.sync_all()?;

        Ok(Self {
            segment_file,
            _index_file: index_file,
            sparse_index_keys,
            sparse_index_offsets,
            buffer: Vec::with_capacity(BUF_LEN),
        })
    }

    pub fn get<V: Decode>(&mut self, k: &K) -> Result<Option<V>, SSTableError> {
        let offset_index = match self.sparse_index_keys.binary_search(k) {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        };

        let offset = self.sparse_index_offsets[offset_index];
        let next_offset = if offset_index == self.sparse_index_offsets.len() - 1 {
            self.segment_file.stream_position()?
        } else {
            self.sparse_index_offsets[offset_index + 1]
        };

        let block_size = (next_offset - offset) as usize;
        let mut compressed_block = vec![0u8; block_size];

        self.segment_file.seek(SeekFrom::Start(offset))?;
        self.segment_file.read_exact(&mut compressed_block)?;

        self.buffer.clear();
        bulk::decompress_to_buffer(&compressed_block, &mut self.buffer)?;

        let mut cursor = Cursor::new(self.buffer.as_slice());

        while (cursor.position() as usize) < self.buffer.len() {
            let key_len = read_usize(&mut cursor)?;
            let mut key_bytes = vec![0u8; key_len];
            cursor.read_exact(&mut key_bytes)?;

            let value_len = read_usize(&mut cursor)?;

            let mut value_bytes = vec![0u8; value_len];
            cursor.read_exact(&mut value_bytes)?;

            let mut key_input: &[u8] = key_bytes.as_slice();
            let key = K::decode_from(&mut key_input)?;

            let mut value_input: &[u8] = value_bytes.as_slice();
            let value = V::decode_from(&mut value_input)?;

            if &key == k {
                self.buffer.clear();
                if value_len == 0 {
                    return Ok(None);
                }
                return Ok(Some(value));
            }
        }

        self.buffer.clear();
        Ok(None)
    }

    pub fn sparse_index_keys(&self) -> Vec<K> {
        self.sparse_index_keys.clone()
    }

    pub fn sparse_index_offsets(&self) -> Vec<u64> {
        self.sparse_index_offsets.clone()
    }
}
