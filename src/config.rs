use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::time::SystemTime;

use toml;

pub const FILENAME: &'static str = "noten.toml";

#[derive(Debug, Deserialize)]
pub struct Directories {
    pub input: String,
    pub output: String,
    pub generators: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub skeleton: String,
    pub index: String,
    pub directories: Directories,
    pub constants: toml::value::Table,
}

quick_error! {
    #[derive(Debug)]
    pub enum ReadError {
        Io(err: io::Error) {
            from()
        }
        De(err: toml::de::Error) {
            from()
        }
    }
}

/// Reads the configuration, returns (config, last-modified).
pub fn read() -> Result<(Config, SystemTime), ReadError> {
    let mut file = try!(File::open(FILENAME));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    let config = toml::from_str(&text)?;
    let time = file.metadata().unwrap().modified().unwrap();
    Ok((config, time))
}
