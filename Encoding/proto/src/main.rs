use crate::fs::File;
use proto::example;
use regex::Regex;
use std::env;
use std::fmt;
use std::fmt::Formatter;
use std::fs;
use std::io;
use std::io::BufWriter;
use std::io::Write;

trait Declaration: fmt::Display {}

trait BinaryEncode {
    fn encode_code(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

trait BinaryDecode: Sized {
    fn decode_code(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

enum Type {
    String,
    Uint32,
    Uint64,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Type::String => "String",
            Type::Uint32 => "u32",
            Type::Uint64 => "u64",
        };
        write!(f, "{s}")
    }
}

impl From<&str> for Type {
    fn from(s: &str) -> Self {
        match s {
            "string" => Type::String,
            "uint32" => Type::Uint32,
            "uint64" => Type::Uint64,
            _ => panic!("Invalid Type"),
        }
    }
}

impl From<Type> for String {
    fn from(t: Type) -> Self {
        match t {
            Type::String => "string".to_string(),
            Type::Uint32 => "uint32".to_string(),
            Type::Uint64 => "uint64".to_string(),
        }
    }
}

impl From<&Type> for u8 {
    fn from(t: &Type) -> Self {
        match t {
            Type::String => 0x00,
            Type::Uint32 => 0x10,
            Type::Uint64 => 0x20,
        }
    }
}

impl From<u8> for Type {
    fn from(u: u8) -> Self {
        match u {
            0x00 => Type::String,
            0x10 => Type::Uint32,
            0x20 => Type::Uint64,
            _ => panic!("Invalid Type"),
        }
    }
}

struct Field {
    typ: Type,
    name: String,
    alias: u8,
}
impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pub {}: {},\n", self.name, self.typ)
    }
}

impl BinaryEncode for Field {
    fn encode_code(&self, f: &mut Formatter) -> fmt::Result {
        let field_number = self.alias;
        let wire_type = match self.typ {
            Type::Uint32 | Type::Uint64 => 0u8,
            Type::String => 2u8,
        };
        let tag = (field_number << 3) | wire_type;

        writeln!(f, "        writer.write_all(&[{tag}])?;")?;

        match self.typ {
            Type::Uint64 => {
                writeln!(
                    f,
                    "        write_varint(self.{} as u64, writer)?;",
                    self.name
                )?;
            }
            Type::Uint32 => {
                writeln!(
                    f,
                    "        write_varint(self.{} as u64, writer)?;",
                    self.name
                )?;
            }
            Type::String => {
                writeln!(
                    f,
                    "        write_varint(self.{}.len() as u64, writer)?;",
                    self.name
                )?;
                writeln!(
                    f,
                    "        writer.write_all(self.{}.as_bytes())?;",
                    self.name
                )?;
            }
        }

        Ok(())
    }
}

impl BinaryDecode for Field {
    fn decode_code(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let field_number = self.alias;
        let wire_type = match self.typ {
            Type::Uint32 | Type::Uint64 => 0u8,
            Type::String => 2u8,
        };
        let tag = (field_number << 3) | wire_type;

        writeln!(f, "            {tag} => {{")?;

        match self.typ {
            Type::Uint64 => {
                writeln!(
                    f,
                    "                msg.{} = read_varint(reader)? as u64;",
                    self.name
                )?;
            }
            Type::Uint32 => {
                writeln!(
                    f,
                    "                msg.{} = read_varint(reader)? as u32;",
                    self.name
                )?;
            }
            Type::String => {
                writeln!(
                    f,
                    "                let len = read_varint(reader)? as usize;"
                )?;
                writeln!(f, "                let mut buf = vec![0u8; len];")?;
                writeln!(f, "                reader.read_exact(&mut buf)?;")?;
                writeln!(
                    f,
                    "                msg.{} = String::from_utf8(buf)",
                    self.name
                )?;
                writeln!(
                    f,
                    "                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, \"invalid utf-8 in {}\"))?;",
                    self.name
                )?;
            }
        }

        writeln!(f, "            }}")?;
        Ok(())
    }
}

struct Message {
    name: String,
    fields: Vec<Field>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "#[derive(Debug, Clone, Default)]")?;
        writeln!(f, "pub struct {} {{", self.name)?;

        for field in &self.fields {
            write!(f, "    {:?}", field)?;
        }

        writeln!(f, "}}")?;

        writeln!(f, "\nimpl BinarySerializable for {} {{", self.name);
        writeln!(
            f,
            "\n   fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {{"
        )?;
        for field in &self.fields {
            field.encode_code(f)?;
        }
        writeln!(f, "        Ok(())")?;
        writeln!(f, "   }}")?;
        writeln!(
            f,
            "   fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {{"
        )?;
        writeln!(f, "        let mut msg = Self::default();")?;
        writeln!(f)?;
        writeln!(f, "        loop {{")?;
        writeln!(f, "            let tag = match read_varint(reader) {{")?;
        writeln!(f, "                Ok(tag) => tag as u8,")?;
        writeln!(
            f,
            "                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,"
        )?;
        writeln!(f, "                Err(e) => return Err(e),")?;
        writeln!(f, "            }};")?;
        writeln!(f)?;
        writeln!(f, "            match tag {{")?;
        for field in &self.fields {
            field.decode_code(f)?;
        }
        writeln!(f, "                _ => {{")?;
        writeln!(
            f,
            "                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!(\"unknown tag: {{}}\", tag)));"
        )?;
        writeln!(f, "                }}")?;
        writeln!(f, "            }}")?;
        writeln!(f, "        }}")?;
        writeln!(f)?;
        writeln!(f, "        Ok(msg)")?;
        writeln!(f, "   }}")?;
        writeln!(f, "}}")
    }
}

impl Declaration for Message {}

struct Fn {
    name: String,
    arg: String,
    ret: String,
}

struct Service {
    name: String,
    rpcs: Vec<Fn>,
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "pub trait {} {{", self.name)?;
        for rpc in &self.rpcs {
            writeln!(f, "   fn {}(r: {}) -> {};", rpc.name, rpc.arg, rpc.ret)?;
        }

        writeln!(f, "}}")
    }
}

impl Declaration for Service {}

fn read_message<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Option<Message> {
    let name = iter.next()?;
    let mut fields: Vec<Field> = Vec::new();

    loop {
        let w = iter.next()?;
        match w {
            "}" => break,
            "{" => (),
            word => {
                let typ = word;
                let name = iter.next()?;
                iter.next()?;
                let alias_word: &str = iter.next()?;
                let alias: u8 = match alias_word[..1].parse() {
                    Ok(a) => a,
                    Err(e) => panic!("Failed to convert string to alias {e} {alias_word}"),
                };
                let field = Field {
                    typ: Type::from(typ),
                    name: String::from(name),
                    alias: alias,
                };
                fields.push(field);
            }
        }
    }

    return Some(Message {
        name: String::from(name),
        fields: fields,
    });
}

fn read_service<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Option<Service> {
    let name = iter.next()?;
    let mut rpcs: Vec<Fn> = Vec::new();

    loop {
        let w = iter.next()?;
        match w {
            "}" => break,
            "{" => (),
            _ => {
                let func_def = iter.next()?;
                let re = Regex::new(r"[()]").expect("Invalid Regex");
                let mut fn_iter = re.split(func_def);
                let fn_name = fn_iter.next()?;
                let fn_arg = fn_iter.next()?;
                iter.next()?;
                let func_def = iter.next()?;
                let mut fn_iter = re.split(func_def);
                fn_iter.next()?;
                let fn_ret = fn_iter.next()?;
                let fn_instance = Fn {
                    name: String::from(fn_name),
                    arg: String::from(fn_arg),
                    ret: String::from(fn_ret),
                };
                rpcs.push(fn_instance);
            }
        }
    }

    return Some(Service {
        name: String::from(name),
        rpcs: rpcs,
    });
}

fn write_trait(writer: &mut BufWriter<File>) -> io::Result<()> {
    writeln!(
        writer,
        "use std::io::{{self, Read, Write}};

pub trait BinarySerializable: Sized {{
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self>;
}}
fn write_varint(mut value: u64, writer: &mut dyn Write) -> io::Result<()> {{
    loop {{
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {{
            byte |= 0x80;
        }}
        writer.write_all(&[byte])?;
        if value == 0 {{
            break;
        }}
    }}
    Ok(())
}}

fn read_varint(reader: &mut dyn Read) -> io::Result<u64> {{
    let mut result: u64 = 0x00;

    loop {{
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let byte = buf[0];
        
        result |= ( byte & 0x7f ) as u64;
        if byte >> 7 == 0 {{
            break;
        }}
        result <<= 7;
    }}

    Ok(result)
}}
"
    )
}

fn write_declarations<T: Declaration>(writer: &mut BufWriter<File>, decs: &[T]) -> io::Result<()> {
    for dec in decs {
        writeln!(writer, "{dec}")?;
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let in_path = &args[1];
    let out_path = &args[2];

    let contents: String =
        fs::read_to_string(in_path).expect("Should have been able to read the file");

    let mut messages: Vec<Message> = Vec::new();
    let mut services: Vec<Service> = Vec::new();

    let mut word_iter = contents.split_whitespace();

    loop {
        let c = word_iter.next();
        match c {
            Some(c) => match c {
                "message" => match read_message(&mut word_iter) {
                    Some(m) => messages.push(m),
                    None => (),
                },
                "service" => match read_service(&mut word_iter) {
                    Some(s) => services.push(s),
                    None => (),
                },
                _ => (),
            },
            None => break,
        }
    }

    if messages.len() == 0 && services.len() == 0 {
        panic!("No Keywords Found");
    }

    let file: File = File::create(out_path).expect("IO ERROR 1");
    let mut writer: BufWriter<File> = BufWriter::new(file);
    write_trait(&mut writer).expect("IO ERROR 2");
    write_declarations(&mut writer, &messages).expect("IO ERROR 3");
    write_declarations(&mut writer, &services).expect("IO ERROR 4");
    writer.flush().expect("IO ERROR 5");

    example::run_example();
}
