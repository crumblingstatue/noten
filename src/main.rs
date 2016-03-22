mod config;

fn main() {
    match config::read() {
        Ok(_) => println!("Okay"),
        Err(e) => {
            println!("Failed opening {} ({}). Not a valid noten project.",
                     config::FILENAME,
                     e)
        }
    }
}
