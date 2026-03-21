use core::{marker::Sized, result::Result};
use std::{
    fs::File,
    io::{BufWriter, IntoInnerError},
};

#[derive(Debug)]
pub enum SSTableError {
    IO(std::io::Error),
    CorruptData,
    IntoInner(IntoInnerError<BufWriter<File>>),
}

pub trait ToLeBytes {
    type Bytes: AsRef<[u8]>;

    fn to_le_bytes(&self) -> Self::Bytes;
}

pub trait Encode {
    fn encode_len(&self) -> usize;
    fn encode_into(&self, buf: &mut Vec<u8>);
}

pub trait Decode: Sized {
    fn decode_from(buf: &mut &[u8]) -> Result<Self, SSTableError>;
}

impl Encode for usize {
    #[inline]
    fn encode_len(&self) -> usize {
        std::mem::size_of::<usize>()
    }

    #[inline]
    fn encode_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encode for u32 {
    #[inline]
    fn encode_len(&self) -> usize {
        4
    }

    #[inline]
    fn encode_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encode for u64 {
    #[inline]
    fn encode_len(&self) -> usize {
        8
    }

    #[inline]
    fn encode_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
impl Encode for i32 {
    #[inline]
    fn encode_len(&self) -> usize {
        4
    }

    #[inline]
    fn encode_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
impl Encode for &str {
    #[inline]
    fn encode_len(&self) -> usize {
        self.as_bytes().len()
    }

    #[inline]
    fn encode_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl Decode for usize {
    fn decode_from(buf: &mut &[u8]) -> Result<Self, SSTableError> {
        let n = std::mem::size_of::<usize>();

        let (head, tail) = buf.split_at(n);
        *buf = tail;

        let mut arr = [0u8; std::mem::size_of::<usize>()];
        arr.copy_from_slice(head);
        Ok(usize::from_le_bytes(arr))
    }
}

impl Decode for u32 {
    fn decode_from(buf: &mut &[u8]) -> Result<Self, SSTableError> {
        let (head, tail) = buf.split_at(4);
        *buf = tail;

        let mut arr = [0u8; 4];
        arr.copy_from_slice(head);
        Ok(u32::from_le_bytes(arr))
    }
}

impl Decode for u64 {
    fn decode_from(buf: &mut &[u8]) -> Result<Self, SSTableError> {
        let (head, tail) = buf.split_at(8);
        *buf = tail;

        let mut arr = [0u8; 8];
        arr.copy_from_slice(head);
        Ok(u64::from_le_bytes(arr))
    }
}

impl Decode for i32 {
    fn decode_from(buf: &mut &[u8]) -> Result<Self, SSTableError> {
        let (head, tail) = buf.split_at(4);
        *buf = tail;

        let mut arr = [0u8; 4];
        arr.copy_from_slice(head);
        Ok(i32::from_le_bytes(arr))
    }
}

impl Decode for String {
    fn decode_from(buf: &mut &[u8]) -> Result<Self, SSTableError> {
        let len = usize::decode_from(buf)?;

        let (head, tail) = buf.split_at(len);
        *buf = tail;

        String::from_utf8(head.to_vec()).map_err(|_| SSTableError::CorruptData)
    }
}
