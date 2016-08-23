
use config::Config;
use process::ProcessingContext;
use regex::{Captures, Regex};
use std::error::Error;
use toml;

fn get_constant_string(name: &str,
                       config: &Config,
                       local_constants: Option<&toml::Table>)
                       -> Result<String, Box<Error>> {
    let constants = &config.constants;
    // Check in local constants first, since they shadow global ones
    if let Some(local) = local_constants {
        if let Some(const_) = local.get(name) {
            return Ok(::util::toml::value_to_string(const_));
        }
    }
    // Now check in global
    match constants.get(name) {
        Some(const_) => Ok(::util::toml::value_to_string(const_)),
        None => Err(format!("Constant `{}` does not exist", name).into()),
    }
}

fn expand_constants(command: &str,
                    config: &Config,
                    local_constants: Option<&toml::Table>)
                    -> Result<String, Box<Error>> {
    let re = Regex::new("%([a-z-]+)").unwrap();
    let mut first_error = None;
    let replaced = re.replace_all(command, |caps: &Captures| {
        let name = caps.at(1).expect("No capture found.");
        match get_constant_string(name, config, local_constants) {
            Ok(c) => c,
            Err(e) => {
                if let None = first_error {
                    first_error = Some(e);
                }
                String::new()
            }
        }
    });
    match first_error {
        None => Ok(replaced),
        Some(err) => Err(err.into()),
    }
}

pub fn substitute<'a>(command: &str,
                      context: &mut ProcessingContext<'a>,
                      local_constants: Option<&toml::Table>)
                      -> Result<String, Box<Error>> {
    let command = try!(expand_constants(command.trim(), context.config, local_constants));
    let re = Regex::new("([a-z]+)(.*)").unwrap();
    let caps = re.captures(&command).unwrap();
    let command = caps.at(1).expect("No command");
    let rest = caps.at(2).expect("No rest");
    debug!("Command: {:?}, Rest: {:?}", command, rest);
    match command {
        "gen" => {
            let caps = re.captures(rest).unwrap();
            let gen_name = &caps[1];
            let rest = &caps[2];
            debug!("Gen: {:?}, Rest: {:?}", gen_name, rest);
            let args = rest.split_whitespace().collect::<Vec<&str>>();
            gen(gen_name, &args, context)
        }
        "url" => Ok(format!("<a href=\"{0}\">{0}</a>", rest.trim())),
        "const" => get_constant_string(rest.trim(), context.config, local_constants),
        _ => Err(format!("Unknown command: {:?}", command).into()),
    }
}

fn gen(gen_name: &str,
       args: &[&str],
       context: &mut ProcessingContext)
       -> Result<String, Box<Error>> {
    use std::process::{Command, Stdio};

    let cfg_generators_dir = context.config
        .generators_dir
        .as_ref()
        .expect("Gen requested but no generators dir");
    let generator_dir = cfg_generators_dir.join(gen_name);
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
