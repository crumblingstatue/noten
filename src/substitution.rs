use crate::config::Config;
use crate::process::ProcessingContext;
use regex::{Captures, Regex};
use std::error::Error;

fn get_constant_string(
    name: &str,
    config: &Config,
    local_constants: Option<&toml::value::Table>,
) -> Result<String, Box<dyn Error>> {
    let constants = &config.constants;
    // Check in local constants first, since they shadow global ones
    if let Some(local) = local_constants {
        if let Some(const_) = local.get(name) {
            return Ok(crate::util::toml::value_to_string(const_));
        }
    }
    // Now check in global
    match constants.get(name) {
        Some(const_) => Ok(crate::util::toml::value_to_string(const_)),
        None => Err(format!("Constant `{}` does not exist", name).into()),
    }
}

fn expand_constants(
    command: &str,
    config: &Config,
    local_constants: Option<&toml::value::Table>,
) -> Result<String, Box<dyn Error>> {
    let re = Regex::new("%([a-z-]+)").unwrap();
    let mut first_error = None;
    let replaced = re.replace_all(command, |caps: &Captures| {
        let name = caps.get(1).expect("No capture found.").as_str();
        match get_constant_string(name, config, local_constants) {
            Ok(c) => c,
            Err(e) => {
                if first_error.is_none() {
                    first_error = Some(e);
                }
                String::new()
            }
        }
    });
    match first_error {
        None => Ok(replaced.into()),
        Some(err) => Err(err),
    }
}

pub fn substitute(
    command: &str,
    context: &mut ProcessingContext,
    local_constants: Option<&toml::value::Table>,
) -> Result<String, Box<dyn Error>> {
    let command = expand_constants(command.trim(), context.config, local_constants)?;
    let re = Regex::new("([a-z]+)(.*)").unwrap();
    let caps = re.captures(&command).unwrap();
    let command = caps.get(1).expect("No command").as_str();
    let rest = caps.get(2).expect("No rest").as_str();
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

fn gen(
    gen_name: &str,
    args: &[&str],
    context: &mut ProcessingContext,
) -> Result<String, Box<dyn Error>> {
    use std::path::Path;
    use std::process::{Command, Stdio};

    let cfg_generators_dir: &Path = context
        .config
        .directories
        .generators
        .as_ref()
        .expect("Gen requested but no generators dir")
        .as_ref();

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
    context
        .template_deps
        .add_dep(context.template_path.to_owned(), gen_cmd_path.to_owned());
    let mut gen_cmd = Command::new(&gen_cmd_path);
    gen_cmd.args(args);
    let output = gen_cmd
        .output()
        .unwrap_or_else(|_| panic!("Failed to spawn {}", gen_name));
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        panic!("{:?} failed.", gen_cmd_path);
    }
}
