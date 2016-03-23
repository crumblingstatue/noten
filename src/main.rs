extern crate toml;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate regex;

mod config;
mod process;
mod substitution;

use config::{Config, ReadError};

use std::fs::{self, File};
use std::io::prelude::*;

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
        let mut file = match File::open(en.path()) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open {:?}: {}", en.path(), e);
                return;
            }
        };
        let mut template = String::new();
        if let Err(e) = file.read_to_string(&mut template) {
            error!("Failed to read template {:?}: {}", en.path(), e);
            return;
        }
        let processed = match process::process(template, &config) {
            Ok(processed) => processed,
            Err(e) => {
                error!("Failed to process template {:?}: {}", en.path(), e);
                return;
            }
        };
        println!("Output:\n------\n{}\n------", processed);
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
