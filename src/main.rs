extern crate env_logger;
extern crate hoedown;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod config;
mod process;
mod skeleton;
mod substitution;
mod template_deps;
mod util;

use config::{Config, ReadError};
use std::fs::{self, File};
use std::io::prelude::*;
use std::time::SystemTime;

/// Returns whether the entry to process is up-to-date, which
/// means it does not need processing.
fn up_to_date(
    template: &fs::Metadata,
    content: Option<&fs::Metadata>,
    exe_modif: &SystemTime,
    config_modif: &SystemTime,
    skel_modif: &SystemTime,
    dep_modifs: &[SystemTime],
) -> bool {
    match content {
        None => false,
        Some(content_meta) => {
            let content_modif = content_meta.modified().unwrap();
            for dep_modif in dep_modifs {
                if *dep_modif > content_modif {
                    return false;
                }
            }
            if *exe_modif > content_modif
                || *config_modif > content_modif
                || *skel_modif > content_modif
            {
                false
            } else {
                let template_modif = template.modified().unwrap();
                content_modif > template_modif
            }
        }
    }
}

fn run(config: &Config, exe_modif: &SystemTime, config_modif: &SystemTime) {
    use process::ProcessingContext;
    use std::path::Path;
    use template_deps::TemplateDeps;

    let (skeleton, skel_modif) = skeleton::Skeleton::parse_file(&config.skeleton).unwrap();

    let mut template_deps = if Path::new(template_deps::PATH).exists() {
        TemplateDeps::open().unwrap()
    } else {
        TemplateDeps::default()
    };

    let entries = match fs::read_dir(&config.directories.input) {
        Ok(entries) => entries,
        Err(e) => {
            error!(
                "Failed to read input directory {:?}: {}",
                config.directories.input, e
            );
            return;
        }
    };
    let mut out_files = Vec::new();
    for en in entries {
        let en = match en {
            Ok(en) => en,
            Err(e) => {
                error!("Failed to read directory entry: {}", e);
                return;
            }
        };

        let path = en.path();
        if path.extension() != Some("noten".as_ref()) {
            warn!(
                "Skipping {:?}, because it doesn't have .noten extension",
                path
            );
            continue;
        }
        debug!("Checking up-to-dateness of {:?}", path);
        let stem = path
            .file_stem()
            .expect("File doesnt' have a stem. The fuck?")
            .to_owned();
        let mut out_filename = stem.clone();
        out_filename.push(".html");
        let out_path = AsRef::<Path>::as_ref(&config.directories.output).join(out_filename);
        out_files.push(out_path.clone());
        let mut dep_modifs = Vec::new();
        if let Some(deps) = template_deps.hash_map.get(&path) {
            for path in deps {
                use std::process::Command;
                match Command::new("cargo")
                    .current_dir(path.parent().unwrap())
                    .arg("build")
                    .arg("--release")
                    .status()
                {
                    Ok(status) if status.success() => {
                        let meta = fs::metadata(path).unwrap();
                        let modif = meta.modified().unwrap();
                        dep_modifs.push(modif);
                    }
                    Ok(status) => {
                        eprintln!("Cargo returned with status: {}", status);
                    }
                    Err(e) => {
                        eprintln!("Cargo spawn error: {}", e)
                    }
                }
            }
        }

        if up_to_date(
            &en.metadata().unwrap(),
            fs::metadata(&out_path).ok().as_ref(),
            exe_modif,
            config_modif,
            &skel_modif,
            &dep_modifs,
        ) {
            info!("{:?} is up to date", &path);
            continue;
        }

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
        let mut context = ProcessingContext {
            template_path: &path,
            template_deps: &mut template_deps,
            config,
        };
        let processed = match process::process(&template, &mut context, &skeleton) {
            Ok(processed) => processed,
            Err(e) => {
                error!("Failed to process template {:?}: {}", &path, e);
                return;
            }
        };
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
        if stem.to_str().expect("Index doc path is not UTF-8") == config.index {
            if let Err(e) = std::fs::copy(&out_path, "index.html") {
                error!("Failed to copy to index.html: {}", e);
                return;
            }
        }
    }
    for en in fs::read_dir(&config.directories.output).unwrap() {
        let path = en.unwrap().path();
        if !out_files.contains(&path) {
            println!("Removing non-generated artifact {:?}", path);
            fs::remove_file(path).unwrap();
        }
    }
    template_deps.save().unwrap();
}

fn main() {
    env_logger::init();

    match config::read() {
        Ok((config, config_modif)) => {
            util::fs::create_dir_if_not_exists(".noten").unwrap();
            let exe_modif = fs::metadata(::std::env::current_exe().unwrap())
                .unwrap()
                .modified()
                .unwrap();
            run(&config, &exe_modif, &config_modif);
        }
        Err(ReadError::Io(err)) => error!(
            "Failed opening {} ({}). Not a valid noten project.",
            config::FILENAME,
            err
        ),
        Err(ReadError::De(err)) => error!("Failed to parse {}: {}", config::FILENAME, err),
    }
}
