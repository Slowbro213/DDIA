use std::env;
use std::fmt;
use std::fs;

enum Type {
    String,
    Uint32,
    Uint64,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Type::String => "String",
            Type::Uint32 => "Uint32",
            Type::Uint64 => "Uint64",
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

struct Field {
    typ: Type,
    name: String,
    alias: u32,
}
impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(type: {}; name: {}; alias: {})",
            self.typ, self.name, self.alias
        )
    }
}

struct Message {
    name: String,
    fields: Vec<Field>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{:?})", self.name, self.fields)
    }
}

struct Fn {
    name: String,
    arg: Message,
    ret: Message,
}
impl fmt::Debug for Fn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(name: {}; arg: {}; ret: {})",
            self.name, self.arg, self.ret
        )
    }
}

struct Service {
    name: String,
    rpcs: Vec<Fn>,
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{:?})", self.name, self.rpcs)
    }
}

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
                let alias: u32 = match alias_word[..1].parse() {
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
    let mut fields: Vec<Field> = Vec::new();

    loop {
        let w = iter.next()?;
        match w {
            "}" => break,
            "{" => (),
            _ => {
                let func_def = iter.next()?;
                let mut fn_iter = func_def.split("(|)");
                let fn_name = fn_iter.next()?;
                fn_iter.next()?;
                let fn_arg = fn_iter.next()?;

                let func_def = iter.next()?;
                let mut fn_iter = func_def.split("(|)");
                fn_iter.next()?;
                let fn_ret = fn_iter.next()?;
                let fn = Fn{ name: String::from(fn_name), arg: fn_arg, ret: fn_ret  };
            }
        }
    }

    return Some(Service {
        name: String::from(name),
        rpcs: vec![],
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = &args[1];

    let contents: String =
        fs::read_to_string(path).expect("Should have been able to read the file");

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

    for m in messages {
        println!("{m}\n");
    }

    for s in services {
        println!("{s}\n");
    }
}
