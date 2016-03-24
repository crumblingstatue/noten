use std::error::Error;
use config::Config;
use regex::{Captures, Regex};
use std::path::Path;
use std::io;
use std::io::prelude::*;

fn expand_constants(command: &str, config: &Config) -> String {
    let re = Regex::new("%([a-z-]+)").unwrap();
    re.replace_all(command, |caps: &Captures| {
        let name = caps.at(1).expect("No capture found.");
        let consts = &config.constants;
        let substitute = consts.get(name).expect("No substitute found.");
        substitute.to_string()
    })
}

pub fn substitute(command: &str, config: &Config) -> Result<String, Box<Error>> {
    let command = expand_constants(command.trim(), config);
    let re = Regex::new("([a-z]+)(.*)").unwrap();
    let caps = re.captures(&command).unwrap();
    let command = caps.at(1).expect("No command");
    let rest = caps.at(2).expect("No rest");
    debug!("Command: {:?}, Rest: {:?}", command, rest);
    match command {
        "gen" => {
            let caps = re.captures(&rest).unwrap();
            let gen_name = &caps[1];
            let rest = &caps[2];
            debug!("Gen: {:?}, Rest: {:?}", gen_name, rest);
            let args = rest.split_whitespace().collect::<Vec<&str>>();
            gen(gen_name,
                &args,
                config.generators_dir.as_ref().expect("Gen requested but no generators dir."))
        }
        _ => panic!("Unknown command: {:?}", command),
    }
}

fn gen(gen_name: &str, args: &[&str], generators_dir: &Path) -> Result<String, Box<Error>> {
    use std::process::Command;

    let generator_dir = generators_dir.join(gen_name);
    if !generator_dir.exists() {
        panic!("{:?} does not exist.", generator_dir);
    }
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&generator_dir)
       .arg("rustc")
       .arg("--release")
       .arg("--color=always")
       .arg("--")
       .arg("--color=always");
    let output = cmd.output().expect("Failed to spawn cargo");
    let _ = io::stdout().write_all(&output.stdout);
    let _ = io::stderr().write_all(&output.stderr);
    if !output.status.success() {
        panic!("cargo failed");
    }
    let gen_cmd_path = generator_dir.join(format!("target/release/{}", gen_name));
    debug!("Gen command path is {:?}", gen_cmd_path);
    let mut gen_cmd = Command::new(&gen_cmd_path);
    gen_cmd.args(args);
    let output = gen_cmd.output().expect(&format!("Failed to spawn {}", gen_name));
    if output.status.success() {
        Ok(try!(String::from_utf8(output.stdout)))
    } else {
        panic!("{:?} failed.", gen_cmd_path);
    }
}