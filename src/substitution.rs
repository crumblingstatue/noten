use std::error::Error;
use config::Config;
use regex::{Captures, Regex};
use std::path::Path;
use process::ProcessingContext;

fn expand_constants(command: &str, config: &Config) -> String {
    let re = Regex::new("%([a-z-]+)").unwrap();
    re.replace_all(command, |caps: &Captures| {
        let name = caps.at(1).expect("No capture found.");
        let consts = &config.constants;
        let substitute = consts.get(name).expect("No substitute found.");
        substitute.to_string()
    })
}

pub fn substitute<'a>(command: &str,
                      config: &Config,
                      context: &mut ProcessingContext<'a>)
                      -> Result<String, Box<Error>> {
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
                config.generators_dir.as_ref().expect("Gen requested but no generators dir."),
                context)
        }
        _ => panic!("Unknown command: {:?}", command),
    }
}

fn gen<'a>(gen_name: &str,
           args: &[&str],
           generators_dir: &Path,
           context: &mut ProcessingContext<'a>)
           -> Result<String, Box<Error>> {
    use std::process::{Command, Stdio};

    let generator_dir = generators_dir.join(gen_name);
    if !generator_dir.exists() {
        panic!("{:?} does not exist.", generator_dir);
    }
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&generator_dir)
       .stdout(Stdio::inherit())
       .arg("build")
       .arg("--release");
    let status = cmd.status().expect("Failed to spawn cargo");
    if !status.success() {
        panic!("cargo failed");
    }
    let gen_cmd_path = generator_dir.join(format!("target/release/{}", gen_name));
    debug!("Gen command path is {:?}", gen_cmd_path);
    context.template_deps.add_dep(context.template_path.to_owned(), gen_cmd_path.to_owned());
    let mut gen_cmd = Command::new(&gen_cmd_path);
    gen_cmd.args(args);
    let output = gen_cmd.output().expect(&format!("Failed to spawn {}", gen_name));
    if output.status.success() {
        Ok(try!(String::from_utf8(output.stdout)))
    } else {
        panic!("{:?} failed.", gen_cmd_path);
    }
}
