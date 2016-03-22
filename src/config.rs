use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::borrow::Cow;

use toml;

pub const FILENAME: &'static str = "noten.toml";

#[derive(Debug)]
pub struct Config {
    input_dir: PathBuf,
    output_dir: PathBuf,
    generators_dir: Option<PathBuf>,
    constants: toml::Table,
}

quick_error! {
    #[derive(Debug)]
    pub enum ReadError {
        Io(err: io::Error) {
            from()
        }
        TomlParser(msg: String) {}
        MissingField(name: Cow<'static, str>) {}
        TypeMismatch(name: &'static str, expected: &'static str, got: &'static str) {}
    }
}

trait ToTomlTable {
    fn to_table(&self) -> Result<&toml::Table, TypeMismatchError>;
}

impl ToTomlTable for toml::Table {
    fn to_table(&self) -> Result<&toml::Table, TypeMismatchError> {
        Ok(self)
    }
}

impl ToTomlTable for toml::Value {
    fn to_table(&self) -> Result<&toml::Table, TypeMismatchError> {
        self.as_table().ok_or(TypeMismatchError {
            expected: "table",
            got: self.type_str(),
        })
    }
}

struct TypeMismatchError {
    expected: &'static str,
    got: &'static str,
}

fn require_field<'a, T: ToTomlTable + 'a>(to_table: &'a T,
                                          name: &'static str)
                                          -> Result<&'a toml::Value, ReadError> {
    let table = try!(to_table.to_table()
                             .map_err(|e| ReadError::TypeMismatch(name, e.expected, e.got)));
    table.get(name).ok_or_else(|| ReadError::MissingField(name.into()))
}

pub fn read() -> Result<Config, ReadError> {
    let mut file = try!(File::open(FILENAME));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    let mut parser = toml::Parser::new(&text);
    let table = match parser.parse() {
        Some(table) => table,
        None => {
            let mut msg = String::new();
            for e in &parser.errors {
                let (lo_line, lo_col) = parser.to_linecol(e.lo);
                let (hi_line, hi_col) = parser.to_linecol(e.hi);
                msg.push_str(&format!("{}:{} -> {}:{} : {}\n",
                                      lo_line + 1,
                                      lo_col + 1,
                                      hi_line + 1,
                                      hi_col + 1,
                                      e.desc));
            }
            return Err(ReadError::TomlParser(msg));
        }
    };
    macro_rules! convert {
        ($p:path, $val:expr) => {
            match $val {
                &$p(ref s) => s.clone(),
                value => {
                    let e = ReadError::TypeMismatch("directories.input",
                                                    "string",
                                                    value.type_str());
                    return Err(e);
                }
            }
        }
    }
    let directories = try!(require_field(&table, "directories"));
    let input_dir = try!(require_field(directories, "input"));
    let input_dir = convert!(toml::Value::String, input_dir).into();
    let output_dir = try!(require_field(directories, "output"));
    let output_dir = convert!(toml::Value::String, output_dir).into();
    let generators_dir = directories.lookup("generators");
    let generators_dir = match generators_dir {
        Some(dir) => Some(convert!(toml::Value::String, dir).into()),
        None => None,
    };
    let constants = match table.get("constants") {
        Some(value) => convert!(toml::Value::Table, value),
        None => toml::Table::new(),
    };
    Ok(Config {
        input_dir: input_dir,
        output_dir: output_dir,
        generators_dir: generators_dir,
        constants: constants,
    })
}
