extern crate toml;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;
extern crate env_logger;

mod config;

use config::ReadError;

fn main() {
    env_logger::init().unwrap();

    match config::read() {
        Ok(config) => println!("{:#?}", config),
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
