use std::fs::File;
use std::error::Error;
use std::io::prelude::*;

pub const FILENAME: &'static str = "noten.toml";

pub struct Config;

pub fn read() -> Result<Config, Box<Error>> {
    let mut file = try!(File::open(FILENAME));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    println!("Config: {}", text);
    Ok(Config)
}
