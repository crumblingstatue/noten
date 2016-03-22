extern crate toml;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;
extern crate env_logger;

mod config;

use config::{Config, ReadError};

use std::fs;

fn run(config: Config) {
    let entries = match fs::read_dir(&config.input_dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read input directory {:?}: {}",
                   config.input_dir,
                   e);
            return;
        }
    };
    for en in entries {
        let en = match en {
            Ok(en) => en,
            Err(e) => {
                error!("Failed to read directory entry: {}", e);
                return;
            }
        };
        println!("Processing {:?}", en.path());
    }
}

fn main() {
    env_logger::init().unwrap();

    match config::read() {
        Ok(config) => run(config),
        Err(ReadError::Io(err)) => {
            error!("Failed opening {} ({}). Not a valid noten project.",
                   config::FILENAME,
                   err)
        }
        Err(ReadError::TomlParser(msg)) => error!("Failed to parse {}:\n{}", config::FILENAME, msg),
        Err(ReadError::MissingField(name)) => error!("Missing required field: {}", name),
        Err(ReadError::TypeMismatch(name, expected, got)) => {
            error!("Field {} should be of type {}, but it is {}",
                   name,
                   expected,
                   got)
        }
    }
}
