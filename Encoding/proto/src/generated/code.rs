use std::io::{self, Read, Write};

pub trait BinarySerializable: Sized {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self>;
}
fn write_varint(mut value: u64, writer: &mut dyn Write) -> io::Result<()> {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        writer.write_all(&[byte])?;
        if value == 0 {
            break;
        }
    }
    Ok(())
}

fn read_varint(reader: &mut dyn Read) -> io::Result<u64> {
    let mut result: u64 = 0x00;
    let mut shift = 0;

    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let byte = buf[0];
        
        result |= (( byte & 0x7f ) as u64) << (shift*7);
        if byte >> 7 == 0 {
            break;
        }
        shift+=1;
    }

    Ok(result)
}

#[derive(Debug, Clone, Default)]
pub struct AddUserRequest {
    pub name: String,
    pub surname: String,
    pub age: u32,
}

impl BinarySerializable for AddUserRequest {

   fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[10])?;
        write_varint(self.name.len() as u64, writer)?;
        writer.write_all(self.name.as_bytes())?;
        writer.write_all(&[18])?;
        write_varint(self.surname.len() as u64, writer)?;
        writer.write_all(self.surname.as_bytes())?;
        writer.write_all(&[24])?;
        write_varint(self.age as u64, writer)?;
        Ok(())
   }
   fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut msg = Self::default();

        loop {
            let tag = match read_varint(reader) {
                Ok(tag) => tag as u8,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            match tag {
            10 => {
                let len = read_varint(reader)? as usize;
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                msg.name = String::from_utf8(buf)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8 in name"))?;
            }
            18 => {
                let len = read_varint(reader)? as usize;
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                msg.surname = String::from_utf8(buf)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8 in surname"))?;
            }
            24 => {
                msg.age = read_varint(reader)? as u32;
            }
                _ => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown tag: {}", tag)));
                }
            }
        }

        Ok(msg)
   }
}

#[derive(Debug, Clone, Default)]
pub struct AddUserResponse {
    pub id: String,
}

impl BinarySerializable for AddUserResponse {

   fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[10])?;
        write_varint(self.id.len() as u64, writer)?;
        writer.write_all(self.id.as_bytes())?;
        Ok(())
   }
   fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut msg = Self::default();

        loop {
            let tag = match read_varint(reader) {
                Ok(tag) => tag as u8,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            match tag {
            10 => {
                let len = read_varint(reader)? as usize;
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                msg.id = String::from_utf8(buf)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8 in id"))?;
            }
                _ => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown tag: {}", tag)));
                }
            }
        }

        Ok(msg)
   }
}

pub trait UserService {
   fn AddUser(r: AddUserRequest) -> AddUserResponse;
}

