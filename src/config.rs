use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;

use toml;

pub const FILENAME: &'static str = "noten.toml";

#[derive(Debug)]
pub struct Config {
    pub skeleton_template: PathBuf,
    pub index_doc: String,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub generators_dir: Option<PathBuf>,
    pub constants: toml::Table,
}

quick_error! {
    #[derive(Debug)]
    pub enum ReadError {
        Io(err: io::Error) {
            from()
        }
        TomlParser(msg: String) {}
        Extract(err: ::util::toml::ExtractError) {
            from()
        }
    }
}

/// Reads the configuration, returns (config, last-modified).
pub fn read() -> Result<(Config, SystemTime), ReadError> {
    let mut file = try!(File::open(FILENAME));
    let mut text = String::new();
    try!(file.read_to_string(&mut text));
    let mut parser = toml::Parser::new(&text);
    let table = match parser.parse() {
        Some(table) => table,
        None => {
            let msg = ::util::toml::parser_error_to_string(&parser);
            return Err(ReadError::TomlParser(msg));
        }
    };
    let root = ::util::toml::Extractor::new(table);
    let skeleton_template = try!(root.require::<String>("skeleton")).into();
    let index_doc = try!(root.require("index"));
    let directories = try!(root.require_table("directories"));
    let input_dir = try!(directories.require::<String>("input")).into();
    let output_dir = try!(directories.require::<String>("output")).into();
    let generators_dir = match directories.optional::<String>("generators") {
        Some(result) => Some(try!(result).into()),
        None => None,
    };
    let constants = match root.optional("constants") {
        Some(result) => try!(result),
        None => toml::Table::new(),
    };
    Ok((Config {
            skeleton_template: skeleton_template,
            index_doc: index_doc,
            input_dir: input_dir,
            output_dir: output_dir,
            generators_dir: generators_dir,
            constants: constants,
        },
        file.metadata().unwrap().modified().unwrap()))
}
