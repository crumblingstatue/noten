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
        let path = en.path();
        println!("Processing {:?}", &path);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open {:?}: {}", &path, e);
                return;
            }
        };
        let mut template = String::new();
        if let Err(e) = file.read_to_string(&mut template) {
            error!("Failed to read template {:?}: {}", &path, e);
            return;
        }
        let processed = match process::process(template, &config) {
            Ok(processed) => processed,
            Err(e) => {
                error!("Failed to process template {:?}: {}", &path, e);
                return;
            }
        };
        let mut stem = path.file_stem().expect("File doesnt' have a stem. The fuck?").to_owned();
        stem.push(".php");
        let out_path = config.output_dir.join(stem);
        let mut file = match File::create(&out_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open {:?}: {}", &out_path, e);
                return;
            }
        };
        if let Err(e) = file.write_all(processed.as_bytes()) {
            error!("Failed to write output {:?}: {}", &out_path, e);
            return;
        }
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
