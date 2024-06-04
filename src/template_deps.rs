use {
    log::debug,
    std::{
        collections::HashMap,
        fs::File,
        io::{self, Read as _, Write as _},
        path::{Path, PathBuf},
    },
};

/// The dependencies of each template
#[derive(Default, Debug)]
pub struct TemplateDeps {
    pub hash_map: HashMap<PathBuf, Vec<PathBuf>>,
}

pub const PATH: &str = ".noten/template-deps.toml";

impl TemplateDeps {
    pub fn open() -> io::Result<Self> {
        let mut f = File::open(PATH)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        let table: toml::value::Table = toml::from_str(&s).unwrap();
        let mut hash_map = HashMap::new();
        for (k, v) in &table {
            let k: PathBuf = k.into();
            hash_map.insert(k.clone(), Vec::new());
            if let toml::Value::Array(ref vec) = *v {
                for p in vec {
                    hash_map.get_mut(&k).unwrap().push(p.as_str().unwrap().into());
                }
            } else {
                panic!("Array expected");
            }
        }
        Ok(TemplateDeps { hash_map })
    }
    pub fn clear_deps(&mut self, template_path: &Path) {
        if let Some(entry) = self.hash_map.get_mut(template_path) {
            entry.clear();
        }
    }
    pub fn add_dep(&mut self, template_path: PathBuf, dep_path: PathBuf) {
        use std::collections::hash_map::Entry;
        debug!("Added dep: {:?} => {:?}", template_path, dep_path);

        match self.hash_map.entry(template_path) {
            Entry::Occupied(mut en) => en.get_mut().push(dep_path),
            Entry::Vacant(place) => {
                place.insert(vec![dep_path]);
            }
        }
    }
    pub fn save(&self) -> io::Result<()> {
        let mut table = toml::value::Table::new();
        for (k, v) in &self.hash_map {
            let tp = k.to_string_lossy().into_owned();
            let mut array = toml::value::Array::new();
            for p in v {
                let dp = p.to_string_lossy().into_owned();
                array.push(toml::Value::String(dp));
            }
            table.insert(tp, toml::Value::Array(array));
        }
        let string = toml::ser::to_string_pretty(&table).unwrap();
        let mut f = File::create(PATH)?;
        f.write_all(string.as_bytes())
    }
}
