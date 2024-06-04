mod config;
mod process;
mod skeleton;
mod substitution;
mod template_deps;
mod util;

use {
    config::{Config, ReadError},
    log::{error, warn},
    std::{
        fs::{self, File},
        io::{Read as _, Write as _},
    },
};

fn run(config: &Config) {
    use {process::ProcessingContext, std::path::Path, template_deps::TemplateDeps};

    let skeleton = skeleton::Skeleton::parse_file(&config.skeleton).unwrap();

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
        let stem = path.file_stem().expect("File doesnt' have a stem. The fuck?").to_owned();
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
        Ok(config) => {
            util::fs::create_dir_if_not_exists(".noten").unwrap();
            run(&config);
        }
        Err(ReadError::Io(err)) => error!(
            "Failed opening {} ({}). Not a valid noten project.",
            config::FILENAME,
            err
        ),
        Err(ReadError::De(err)) => error!("Failed to parse {}: {}", config::FILENAME, err),
    }
}
