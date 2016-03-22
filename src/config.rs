use std::fs::File;
use std::io;
use std::io::prelude::*;
use toml;

pub const FILENAME: &'static str = "noten.toml";

pub struct Config;

quick_error! {
    #[derive(Debug)]
    pub enum ReadError {
        Io(err: io::Error) {
            from()
        }
        TomlParser(msg: String) {}
    }
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
                msg.push_str(&format!("{}:{} -> {}:{} : {}",
                                      lo_line + 1,
                                      lo_col + 1,
                                      hi_line + 1,
                                      hi_col + 1,
                                      e.desc));
            }
            return Err(ReadError::TomlParser(msg));
        }
    };
    println!("{:#?}", table);
    Ok(Config)
}
