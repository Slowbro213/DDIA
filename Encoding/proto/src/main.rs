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

fn read_message(contents: &str, offset: usize) -> Message {
    return Message {
        name: String::from("Test"),
        fields: vec![Field {
            typ: Type::String,
            name: String::from("thanas"),
            alias: 1,
        }],
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = &args[1];

    let contents: String =
        fs::read_to_string(path).expect("Should have been able to read the file");

    let mut messages: Vec<Message> = Vec::new();

    let length = contents.len();
    let mut i: usize = 0;
    while i + 7 < length - 1 {
        let keyword = &contents[i..i + 7];
        match keyword {
            "service" => println!("Found select\n"),
            "message" => messages.push(read_message(&contents[i..], i)),
            _ => (),
        }
        i += 1;
    }

    for message in messages {
        println!("{message}\n");
    }
}
