use std::{fs, io};
use std::path::Path;

pub fn create_dir_if_not_exists<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    use std::io::ErrorKind;

    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        },
    }
}
