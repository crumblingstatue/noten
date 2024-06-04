use {
    quick_error::quick_error,
    serde_derive::Deserialize,
    std::{
        fs::File,
        io::{self, prelude::*},
        time::SystemTime,
    },
};

pub const FILENAME: &str = "noten.toml";

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
    let mut file = File::open(FILENAME)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let config = toml::from_str(&text)?;
    let time = file.metadata().unwrap().modified().unwrap();
    Ok((config, time))
}
